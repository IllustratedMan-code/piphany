use std::collections::HashMap;

use super::{
    Dataframe, DataframeCsv, DataframeDB, Derivation, DerivationHash,
    DisplayTable,
};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{ContentArrangement, Table};
use polars::prelude::*;
use polars_utils::aliases::PlSeedableRandomStateQuality;
use sha2::Digest;
use steel::SteelErr;
use steel::SteelVal;
use steel::rvals::FromSteelVal;
use steel::steel_vm::builtin::BuiltInModule;
use steel::steel_vm::register_fn::RegisterFn;

use steel::rvals::Custom;

impl DataframeDB {
    pub fn display(&self) -> DisplayTable {
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            //.set_width(40)
            .add_row(vec!["hash".to_string(), format!("{}", self.hash)]);

        DisplayTable { table }
    }
}

impl Dataframe {
    pub fn display(&self) -> Result<DisplayTable, String> {
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            //.set_width(40)
            .add_row(vec!["hash".to_string(), format!("{}", self.hash()?)]);

        Ok(DisplayTable { table })
    }

    pub fn new(map: HashMap<String, Vec<SteelVal>>) -> Result<Self, SteelErr> {
        let mut columns = vec![];
        for (key, val) in map.into_iter() {
            columns.push(coerce_steel_vec_to_polars_column(key, val)?)
        }
        let frame = DataFrame::new_infer_height(columns).map_err(|x| {
            SteelErr::new(steel::rerrs::ErrorKind::Generic, x.to_string())
        })?;
        Ok(Self { frame })
    }

    pub fn read_csv(path: String) -> Result<Self, String> {
        let frame = CsvReadOptions::default()
            .with_has_header(true)
            .try_into_reader_with_file_path(Some(std::path::PathBuf::from(
                path,
            )))
            .map_err(|x| x.to_string())?
            .finish()
            .map_err(|x| x.to_string())?;
        Ok(Self { frame })
    }

    pub fn hash(&self) -> Result<DerivationHash, String> {
        let mut hasher = sha2::Sha256::new();
        let frame_hash = hash_frame(&self.frame)?.0;
        hasher.update(frame_hash);
        // TODO
        // need to search for derivations in columns and hash those into it.
        // need to scan for custom type columns that have derivations
        let result = hasher.finalize();
        let hash = DerivationHash(format!("{:x}", result));
        Ok(hash)
    }

    pub fn derivations(&self) -> Vec<DerivationHash>{
        vec![DerivationHash("Uninplemented".to_string())]
        //TODO
    }

    pub fn with_column(
        mut self,
        name: String,
        mut values: Vec<SteelVal>,
    ) -> Result<Dataframe, SteelErr> {
        if values.len() == 1 {
            let length = self.frame.shape().0;
            values.resize(length, values[0].clone()) // this prevents panic
        }
        let column = coerce_steel_vec_to_polars_column(name, values)?;
        self.frame.with_column(column).map_err(|x| { // with column broadcasts unit length columns
            SteelErr::new(steel::rerrs::ErrorKind::TypeMismatch, x.to_string())
        })?;

        Ok(self)
    }

    pub fn select(mut self, columns: Vec<String>) -> Result<Self, String> {
        self.frame = self.frame.select(columns).map_err(|x| x.to_string())?;
        Ok(self)
    }

    pub fn as_csv(
        self,
        delimiter: String,
        extension: String,
    ) -> Result<Derivation, String> {
        let mut hasher = sha2::Sha256::new();
        hasher.update(&extension);
        hasher.update(&self.clone().hash()?.0);
        hasher.update(&delimiter);
        let result = hasher.finalize();
        let hash = DerivationHash(format!("{:x}", result));
        let derivations = self.derivations();

        Ok(Derivation::DataframeCsv(DataframeCsv {
            hash,
            frame: self,
            delimiter,
            ext: extension,
            inward_edges: derivations
        }))
    }

    pub fn subset(mut self, expression: String) -> Result<Dataframe, SteelErr> {
        let expression = format!("({})", expression);
        let lexes = subset_lexer(expression);
        let parsed = SubsetParser::new(lexes.clone()?).parse();
        let expr = subset_exec(&self, parsed?);
        let lazy_df = self.frame.lazy();
        self.frame = lazy_df.filter(expr?).collect().map_err(|x| {
            SteelErr::new(steel::rerrs::ErrorKind::Generic, x.to_string())
        })?;
        // need to add hashing here
        Ok(self)
    }
}

pub fn coerce_steel_vec_to_polars_column(
    name: String,
    values: Vec<SteelVal>,
) -> Result<Column, SteelErr> {
    let first = std::mem::discriminant(values.first().ok_or_else(|| {
        SteelErr::new(
            steel::rerrs::ErrorKind::TypeMismatch,
            "Column must be at least length 1!".into(),
        )
    })?);

    let all_same_type =
        values.iter().all(|x| first == std::mem::discriminant(x));

    if !all_same_type {
        return Err(SteelErr::new(
            steel::rerrs::ErrorKind::TypeMismatch,
            "All elements in column must be the same type!".to_string(),
        ));
    }

    let column = match values[0] {
        SteelVal::BoolV(_) => {
            let vals: Vec<bool> = values
                .into_iter()
                .map(|v| {
                    if let SteelVal::BoolV(b) = v {
                        b
                    } else {
                        unreachable!("Already checked all same type")
                    }
                })
                .collect();
            Column::from(Series::new(name.into(), vals))
        }

        SteelVal::IntV(_) => {
            let vals: Vec<i64> = values
                .into_iter()
                .map(|v| {
                    if let SteelVal::IntV(b) = v {
                        b as i64
                    } else {
                        unreachable!("Already checked all same type")
                    }
                })
                .collect();

            Column::from(Series::new(name.into(), vals))
        }

        SteelVal::StringV(_) => {
            let vals: Vec<String> = values
                .into_iter()
                .map(|v| {
                    if let SteelVal::StringV(b) = v {
                        b.as_str().into()
                    } else {
                        unreachable!("Already checked all same type")
                    }
                })
                .collect();

            Column::from(Series::new(name.into(), vals))
        }

        SteelVal::CharV(_) => {
            let vals: Vec<String> = values
                .into_iter()
                .map(|v| {
                    if let SteelVal::CharV(c) = v {
                        c.into()
                    } else {
                        unreachable!("Already checked all same type")
                    }
                })
                .collect();

            Column::from(Series::new(name.into(), vals))
        }

        SteelVal::NumV(_) => {
            let vals: Vec<f64> = values
                .into_iter()
                .map(|v| {
                    if let SteelVal::NumV(c) = v {
                        c
                    } else {
                        unreachable!("Already checked all same type")
                    }
                })
                .collect();

            Column::from(Series::new(name.into(), vals))
        }
        // probably need to handle this element by element instead,
        // could probably wrap it with a template that allows void
        // can use option here?

        SteelVal::Custom(_) => {
            let v: Result<Vec<DerivationHash>, SteelErr> = values
                .into_iter()
                .map(|x| -> Result<DerivationHash, SteelErr> {
                    Ok(Derivation::from_steelval(&x)?.hash())
                })
                .collect();
            let v = v?;

            // This creates a panic if the vector is a single element,
            // due to some broadcasting behavior
            // The panic only occurs within a format! call for some reason (including println!)
            // See https://github.com/pola-rs/polars/issues/27078
            // for now, need to just do the same broadcasting behavior
            // that the Column::from(Series) does
            // to prevent the panic (This is checked for in the with-column function)
            // might want to instead do something expected when passing one element

            let data =
                ObjectChunked::<DerivationHash>::new_from_vec(name.into(), v);
            data.into_column()
        }
        _ => {
            return Err(SteelErr::new(
                steel::rerrs::ErrorKind::TypeMismatch,
                "Unsupported data type for Dataframe".to_string(),
            ));
        }
    };
    Ok(column)
}

pub fn register_steel_functions(module: &mut BuiltInModule) {
    module.register_fn("df::read-csv", Dataframe::read_csv);
    module.register_fn("df::with-column", Dataframe::with_column);
    module.register_fn("df::display", Dataframe::display);
    module.register_fn("df::select", Dataframe::select);
    module.register_fn("df::subset", Dataframe::subset);
    module.register_fn("df::as-csv", Dataframe::as_csv);
    module.register_fn("df::new", Dataframe::new);
}

#[derive(Debug, Clone)]
enum SubsetToken {
    String(String),
    Column(String),
    Number(f64),
    Gt,
    Lt,
    GtEq,
    LtEq,
    Eq,
    And,
    Or,
    LParen,
    RParen,
}

fn subset_lexer(input: String) -> Result<Vec<SubsetToken>, SteelErr> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            c if c.is_whitespace() => {
                chars.next();
            }
            '&' => {
                tokens.push(SubsetToken::And);
                chars.next();
            }
            '|' => {
                tokens.push(SubsetToken::Or);
                chars.next();
            }
            '(' => {
                tokens.push(SubsetToken::LParen);
                chars.next();
            }
            ')' => {
                tokens.push(SubsetToken::RParen);
                chars.next();
            }
            '<' => {
                chars.next();
                if let Some(c) = chars.peek() {
                    if let '=' = c {
                        tokens.push(SubsetToken::LtEq);
                        chars.next();
                    } else {
                        tokens.push(SubsetToken::Lt)
                    }
                } else {
                    return Err(SteelErr::new(
                        steel::rerrs::ErrorKind::BadSyntax,
                        "Nothing follows comparison operator <".to_string(),
                    ));
                }
            }
            '>' => {
                chars.next();
                if let Some(c) = chars.peek() {
                    if let '=' = c {
                        tokens.push(SubsetToken::GtEq);
                        chars.next();
                    } else {
                        tokens.push(SubsetToken::Gt)
                    }
                } else {
                    return Err(SteelErr::new(
                        steel::rerrs::ErrorKind::BadSyntax,
                        "Nothing follows comparison operator >".to_string(),
                    ));
                }
            }
            '=' => {
                chars.next();
                if let Some('=') = chars.peek() {
                    tokens.push(SubsetToken::Eq);
                    chars.next();
                } else {
                    return Err(SteelErr::new(
                        steel::rerrs::ErrorKind::BadSyntax,
                        "Found '=' did you mean '==' ?".to_string(),
                    ));
                }
            }
            '\"' => {
                let mut token = "".to_string();
                let mut closed = false;
                chars.next();
                while let Some(&c) = chars.peek() {
                    chars.next();
                    if c == '\"' {
                        closed = true;
                        break;
                    }
                    token.push(c);
                }
                if !closed {
                    return Err(SteelErr::new(
                        steel::rerrs::ErrorKind::BadSyntax,
                        "String has an unclosed \"".to_string(),
                    ));
                }
                tokens.push(SubsetToken::String(token));
            }
            '\'' => {
                let mut token = "".to_string();
                let mut closed = false;
                chars.next();
                while let Some(&c) = chars.peek() {
                    chars.next();
                    if c == '\'' {
                        closed = true;
                        break;
                    }
                    token.push(c);
                }
                if !closed {
                    return Err(SteelErr::new(
                        steel::rerrs::ErrorKind::BadSyntax,
                        "Column has an unclosed \'".to_string(),
                    ));
                }
                tokens.push(SubsetToken::Column(token));
            }

            c if c.is_alphanumeric() | (c == '_') | (c == '-') => {
                let mut token = "".to_string();
                while let Some(&c) = chars.peek() {
                    if !(c.is_alphanumeric() | (c == '_') | (c == '-')) {
                        break;
                    }
                    chars.next();
                    token.push(c);
                }
                if let Ok(v) = token.parse::<f64>() {
                    tokens.push(SubsetToken::Number(v));
                } else {
                    tokens.push(SubsetToken::Column(token));
                }
            }

            _ => {
                return Err(SteelErr::new(
                    steel::rerrs::ErrorKind::UnexpectedToken,
                    "Unexpected Token in subset".to_string(),
                ));
            }
        }
    }

    Ok(tokens)
}

#[derive(Debug, Clone)]
enum SubsetExpr {
    Value(SubsetToken),
    Column(SubsetToken),
    Op(Box<SubsetExpr>, SubsetToken, Box<SubsetExpr>),
}

struct SubsetParser {
    tokens: Vec<SubsetToken>,
    pos: usize,
}

impl SubsetParser {
    fn new(tokens: Vec<SubsetToken>) -> Self {
        SubsetParser { tokens, pos: 0 }
    }
    fn next(&mut self) -> Option<&SubsetToken> {
        self.pos += 1;
        self.tokens.get(self.pos - 1)
    }
    fn peek(&self) -> Option<SubsetToken> {
        self.tokens.get(self.pos).cloned() // cloned clones the inside of an option
    }
    fn parse_expr(&mut self) -> Result<SubsetExpr, SteelErr> {
        let mut left = self.parse();
        loop {
            let op = match self.peek() {
                Some(SubsetToken::Gt)
                | Some(SubsetToken::Lt)
                | Some(SubsetToken::GtEq)
                | Some(SubsetToken::LtEq)
                | Some(SubsetToken::Eq)
                | Some(SubsetToken::And)
                | Some(SubsetToken::Or) => self.peek(),
                Some(SubsetToken::RParen) => None,
                Some(t) => {
                    return Err(SteelErr::new(
                        steel::rerrs::ErrorKind::UnexpectedToken,
                        format!("unexpected {:?}", t),
                    ));
                }
                _ => None,
            };
            if op.is_none() {
                break;
            }
            if let Some(o) = op {
                self.next(); // eat op

                let right = self.parse()?;
                left = Ok(SubsetExpr::Op(
                    Box::new(left?),
                    o.clone(),
                    Box::new(right),
                ))
            }
        }

        left
    }
    fn parse(&mut self) -> Result<SubsetExpr, SteelErr> {
        // if (, eat, then ), the parse again
        let expr: Result<SubsetExpr, SteelErr>;
        if let Some(t) = self.peek() {
            match t {
                SubsetToken::LParen => {
                    self.next(); // eat '('
                    let e = self.parse_expr();
                    self.next();
                    expr = e;
                }
                SubsetToken::Column(_) => {
                    expr = Ok(SubsetExpr::Column(t.clone()));
                    self.next();
                }
                SubsetToken::String(_) | SubsetToken::Number(_) => {
                    expr = Ok(SubsetExpr::Value(t.clone()));
                    self.next();
                }
                t => {
                    expr = Err(SteelErr::new(
                        steel::rerrs::ErrorKind::UnexpectedToken,
                        format!("Unexpected: {:?}", t),
                    ));
                }
            }
        } else {
            expr = Err(SteelErr::new(
                steel::rerrs::ErrorKind::BadSyntax,
                "nothing to parse!".to_string(),
            ))
        }

        expr
    }
}

fn subset_exec(df: &Dataframe, ast: SubsetExpr) -> Result<Expr, SteelErr> {
    match ast {
        SubsetExpr::Value(v) => match v {
            SubsetToken::Number(n) => Ok(lit(n)),
            SubsetToken::String(s) => Ok(lit(s)),
            _ => Err(SteelErr::new(
                steel::rerrs::ErrorKind::UnexpectedToken,
                format!("Unexpected: {:?}", v),
            )),
        },
        SubsetExpr::Column(c) => match c {
            SubsetToken::Column(c) => Ok(Expr::Column(c.into())),
            _ => Err(SteelErr::new(
                steel::rerrs::ErrorKind::UnexpectedToken,
                format!("Unexpected: {:?}", c),
            )),
        },
        SubsetExpr::Op(x, op, y) => {
            let left = subset_exec(df, *x)?;
            let right = subset_exec(df, *y)?;
            match op {
                SubsetToken::Gt => Ok(left.gt(right)),
                SubsetToken::GtEq => Ok(left.gt_eq(right)),
                SubsetToken::Lt => Ok(left.lt(right)),
                SubsetToken::LtEq => Ok(left.lt_eq(right)),
                SubsetToken::Eq => Ok(left.eq(right)),
                _ => Err(SteelErr::new(
                    steel::rerrs::ErrorKind::UnexpectedToken,
                    format!("Unexpected: {:?}", op),
                )),
            }
        }
    }
}

// looks like polars will work with custom types
// fn test_polars() {
//     let data = [
//         DerivationHash("hi".to_string()),
//         DerivationHash("there".to_string()),
//     ];

//     // undocumented bullshit
//     let s = ObjectChunked::<DerivationHash>::new_from_vec(
//         "my_col".into(),
//         data.into(),
//     );

//     let df = DataFrame::new_infer_height(vec![s.into_column()]).expect("blah");
//     let mut iter = df.columns().iter();
//     let first = iter.next().expect("a key");
// }

fn hash_frame(frame: &DataFrame) -> Result<DerivationHash, String> {
    let mut columns = frame.columns().iter();
    let first = columns.next().ok_or_else(|| {
        "At least one column must exist for hashing".to_string()
    })?;
    let hasher = PlSeedableRandomStateQuality::default();
    let mut hashes = Vec::<u64>::new();

    for col in columns {
        col.vec_hash(hasher.clone(), &mut hashes)
            .map_err(|x| x.to_string())?;
    }

    Ok(DerivationHash(
        hashes
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(""),
    ))
}

// Stuff needed for custom types in polars
impl Default for DerivationHash {
    fn default() -> Self {
        DerivationHash("".to_string())
    }
}

impl polars_utils::total_ord::TotalHash for DerivationHash {
    fn tot_hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        state.write(self.0.as_bytes())
    }
}
impl polars_utils::total_ord::TotalEq for DerivationHash {
    fn tot_eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PolarsObject for DerivationHash {
    fn type_name() -> &'static str {
        "DerivationHash"
    }
}

impl Custom for Dataframe {
    fn fmt(&self) -> Option<std::result::Result<String, std::fmt::Error>> {
        Some(Ok(format!("{}", self.frame)))
    }
}
impl Custom for SubsetToken {
    fn fmt(&self) -> Option<std::result::Result<String, std::fmt::Error>> {
        Some(Ok(format!("{:?}", self)))
    }
}

impl Custom for SubsetExpr {
    fn fmt(&self) -> Option<std::result::Result<String, std::fmt::Error>> {
        let mut s: String = "".into();
        match self.clone() {
            SubsetExpr::Value(v) => s = format!("(Value {:?})", v),
            SubsetExpr::Column(v) => s = format!("(Column {:?})", v),
            SubsetExpr::Op(x, op, y) => {
                s = format!("Op({:?}, {:?}, {:?})", x, op, y)
            }
        }

        Some(Ok(s))
    }
}
