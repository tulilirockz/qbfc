use clap::{Parser, ValueEnum};
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
mod ir;
mod token;
use ir::*;
use token::*;

#[derive(ValueEnum, Clone, PartialEq)]
enum OutputType {
    Binary,
    Sst,
    Asm,
    Ast,
    Debug,
}

#[derive(Parser)]
struct Cli {
    file: String,
    #[clap(
        long,
        default_value_t = false,
        help = "Do not check for valid brainfuck program"
    )]
    no_check: bool,

    #[clap(
        short,
        long,
        default_value = "binary",
        help = "Type of output, either AST, ASM, SST or BINARY"
    )]
    r#type: OutputType,

    #[clap(
        short,
        long,
        default_value = "binary",
        help = "Where the output will be put, - for stdout"
    )]
    output: String,
}

fn main() {
    let args = Cli::parse();

    let compressed_tokens: Vec<CompressedBrainfuckToken> = fs::read(args.file)
        .expect("Failed reading file")
        .iter()
        .filter_map(|x| {
            if x.is_valid_token() {
                Some(Into::<BrainfuckToken>::into(*x))
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .compress()
        .clean();

    if !args.no_check {
        if !compressed_tokens.validate() {
            panic!("Failed due to bracket mismatch");
        }
    }

    let mut bf_prog: QBEIr = QBEIr::new();
    bf_prog
        .init_body()
        .token_array_to_qbe_ir(compressed_tokens)
        .close_prog();

    match args.r#type {
        OutputType::Ast => {
            if args.output == "-" {
                println!("{:#?}", bf_prog.program);
                return;
            }
            fs::write(args.output, format!("{:#?}", bf_prog.program)).expect("Failed writing file");
        }
        OutputType::Sst => {
            if args.output == "-" {
                println!("{}", bf_prog.program);
                return;
            }
            std::fs::write(args.output, format!("{}", bf_prog.program))
                .expect("Failed writing file");
        }
        OutputType::Asm => {
            let mut qbeproc = Command::new("qbe")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .expect("Failure finding QBE binary");
            qbeproc
                .stdin
                .as_mut()
                .unwrap()
                .write_all(format!("{}", bf_prog.program).as_bytes())
                .unwrap();

            if args.output == "-" {
                println!(
                    "{}",
                    String::from_utf8(qbeproc.wait_with_output().unwrap().stdout).unwrap()
                );
                return;
            }
            fs::write(
                args.output,
                format!(
                    "{}",
                    String::from_utf8(qbeproc.wait_with_output().unwrap().stdout).unwrap()
                ),
            )
            .expect("Failed writing file");
        }
        OutputType::Binary => {
            let qbeproc = Command::new("qbe")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .expect("Failure finding QBE binary");
            qbeproc
                .stdin
                .unwrap()
                .write_all(format!("{}", bf_prog.program).as_bytes())
                .unwrap();
            let ccproc = Command::new("cc")
                .stdout(Stdio::piped())
                .stdin(Stdio::from(qbeproc.stdout.unwrap()))
                .args(vec![
                    "-OFast",
                    "-x",
                    "assembler",
                    "-",
                    "-v",
                    "-o",
                    args.output.as_str(),
                ])
                .spawn()
                .expect("Failure finding any C compiler through cc");

            println!(
                "{}",
                String::from_utf8(ccproc.wait_with_output().unwrap().stdout).unwrap()
            );
            return;
        }
        OutputType::Debug => (),
    }
}
