use std::io::Write;
use std::{fs, path::PathBuf};
use steel::steel_vm::engine::Engine;

pub trait Runner {
    fn run_file_or_print_error(&mut self, path: PathBuf)
    -> std::io::Result<()>;
    fn run_builtin_or_print_error(
        &mut self,
        file_contents: &str,
        path: &str,
    ) -> std::io::Result<()>;
}

impl Runner for Engine {
    fn run_file_or_print_error(
        &mut self,
        path: PathBuf,
    ) -> std::io::Result<()> {
        let file_contents = fs::read_to_string(path.clone())?;

        let res = self
            .compile_and_run_raw_program_with_path(file_contents, path.clone());
        match res {
            Ok(_) => (),
            Err(e) => {
                self.raise_error(e);
                panic!("Couldn't run {:?}", path)
            }
        };
        let _ = std::io::stdout().flush();

        Ok(())
    }

    fn run_builtin_or_print_error(
        &mut self,
        file_contents: &str,
        path: &str,
    ) -> std::io::Result<()> {
        let res = self.run(
            file_contents.to_string(),
        );
        match res {
            Ok(_) => (),
            Err(e) => {
                self.raise_error(e);
                panic!("Couldn't run builtin/{:?}", path)
            }
        };
        std::io::stdout().flush()?;

        Ok(())
    }
}
