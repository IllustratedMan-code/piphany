use std::{
    borrow::BorrowMut,
    fs,
    io::Write,
    process::{Command, ExitStatus},
};

use enum_dispatch::enum_dispatch;

enum CacheState {
    Valid,
    Invalid,
}

fn make_dir_check_hash(work_dir: String) -> CacheState {
    match fs::create_dir_all(work_dir.clone()) {
        Ok(_) => CacheState::Invalid, // should check for completeness as well
        Err(e) => match e.kind() {
            std::io::ErrorKind::PermissionDenied => {
                panic!("Cannot create {}, permission denied", work_dir.clone())
            }
            std::io::ErrorKind::AlreadyExists => {
                if std::path::PathBuf::from(format!("{}/.finished", work_dir))
                    .exists()
                {
                    CacheState::Valid
                } else {
                    CacheState::Invalid
                }
            } // this is broekn with create_dir_all i think,
            v => {
                panic!("Couldn't create {} for reason: {}", work_dir, v)
            }
        },
    }
}

fn symlink_edges(
    edges: Vec<super::DerivationHash>,
    all_work_dir: String,
    work_dir: String,
) -> std::io::Result<()> {
    if edges.is_empty() {
        return Ok(());
    } else {
        for i in edges {

            let symlink = std::os::unix::fs::symlink(
                std::path::absolute(format!("{}/{}/out", all_work_dir, i))
                    .expect("couldn't resolve absolute path"),
                std::path::absolute(format!("{}/{}", work_dir, i))
                    .expect("couldn't resolve absolute path"),
            );
            if let Err(e) = symlink {
                match e.kind() {
                    std::io::ErrorKind::AlreadyExists => {
                    },
                    _ => {return Err(e)}
                }
            }
        }
    }

    Ok(())
}

pub fn run_derivation(derivation: &super::Process) -> Option<HPCRuntime> {
    
    
    let work_dir = format!(
        "{}/{:?}/run",
        derivation.work_dir.clone(),
        derivation.hash
    );

    let cache_state = make_dir_check_hash(work_dir.clone());
    symlink_edges(
        derivation.inward_edges.clone(),
        derivation.work_dir.clone(),
        work_dir.clone(),
    )
    .expect("couldn't create symlinks");

    if let CacheState::Valid = cache_state {
        return None;
    }
    let container_runtime: ContainerRuntime;

    if derivation.container.is_none() || derivation.container_runtime.is_none()
    {
        container_runtime = ContainerRuntime::None(NoContainerRuntime::new());
    } else {
        // fill in with runtimes later
        container_runtime = ContainerRuntime::None(NoContainerRuntime::new());
    }

    let mut cmd = derivation.script();
    let mut hpc_r = NoHPCRuntime::new();
    cmd = container_runtime.cmd(cmd);
    hpc_r.submit_job(derivation.shell.clone(), cmd, work_dir.clone());
    Some(HPCRuntime::from(hpc_r))
}

#[enum_dispatch(HPCRuntimeFunctions)]
pub enum HPCRuntime {
    NoHPCRuntime,
}

#[enum_dispatch]
pub trait HPCRuntimeFunctions {
    fn submit_job(&mut self, shell: String, cmd: String, work_dir: String);
    fn cmd(&self, cmd: String) -> String;
    fn wait(&mut self) -> Option<ExitStatus>;
    fn finished(&mut self) -> bool;
}

pub struct NoHPCRuntime {
    childprocess: Option<std::process::Child>,
}

impl NoHPCRuntime {
    fn new() -> Self {
        Self { childprocess: None }
    }
}

fn write_command_to_file(cmd: String, work_dir: String) -> std::io::Result<()> {
    let mut file = fs::File::create(format!("{}/.cmd", work_dir))?;
    file.write_all(&cmd.into_bytes())?;
    Ok(())
}

impl HPCRuntimeFunctions for NoHPCRuntime {
    fn submit_job(&mut self, shell: String, cmd: String, work_dir: String) {
        let cmd = self.cmd(cmd);
        write_command_to_file(cmd.clone(), work_dir.clone())
            .expect("couldn't write cmd to file");
        let mut child = Command::new("sh");
        self.childprocess = Some(
            child
                .arg(".cmd")
                .current_dir(work_dir)
                .spawn()
                .unwrap_or_else(|e| {
                    panic!(
                        "couldn't start process: {} because of: {}",
                        cmd.clone(),
                        e
                    )
                }),
        );
    }
    fn cmd(&self, cmd: String) -> String {
        cmd
    }
    fn wait(&mut self) -> Option<ExitStatus> {
        Some(
            self.childprocess
                .take()?
                .wait()
                .expect("failed to wait for job"),
        )
    }
    fn finished(&mut self) -> bool {
        if let Some(c) = self.childprocess.borrow_mut() {
            match c.try_wait() {
                Ok(Some(status)) => true,
                Ok(None) => false,
                Err(e) => panic!("failed to check job status"),
            }
        } else {
            false // hasn't started yet
        }
    }
}

pub struct LsfHPCRuntime {}

#[enum_dispatch(ContainerRuntimeFunctions)]
pub enum ContainerRuntime {
    None(NoContainerRuntime),
}

#[enum_dispatch]
pub trait ContainerRuntimeFunctions {
    fn cmd(&self, cmd: String) -> String;
}

pub struct NoContainerRuntime {}

impl NoContainerRuntime {
    fn new() -> Self {
        Self {}
    }
}

impl ContainerRuntimeFunctions for NoContainerRuntime {
    fn cmd(&self, cmd: String) -> String {
        cmd
    }
}
