use clap::{ArgEnum, Parser};
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
enum OutputType {
    Binary,
    Qbe,
    Ast,
}

#[derive(Parser)]
struct Cli {
    file: String,
    #[clap(long, default_value_t = false)]
    no_check: bool,
    #[clap(long, default_value_t = false)]
    no_clean: bool,

    #[clap(arg_enum)]
    output: OutputType,

    #[clap(short, long, default_value_t = false)]
    unroll_loops: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum BrainfuckToken {
    Next,
    Prev,
    Add,
    Sub,
    Out,
    Input,
    LoopStart,
    LoopEnd,
}

impl BrainfuckToken {
    fn to_opposite(self: Self) -> Option<BrainfuckToken> {
        match self {
            BrainfuckToken::Next => Some(Self::Prev),
            BrainfuckToken::Prev => Some(Self::Next),
            BrainfuckToken::Add => Some(Self::Sub),
            BrainfuckToken::Sub => Some(Self::Add),
            BrainfuckToken::LoopStart => Some(Self::LoopEnd),
            BrainfuckToken::LoopEnd => Some(Self::LoopStart),
            BrainfuckToken::Input => None,
            BrainfuckToken::Out => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct CompressedBrainfuckToken {
    token: BrainfuckToken,
    num: u64,
}

struct WhileLoop {
    join_num: usize,
}

fn main() {
    let args = Cli::parse();
    if args.unroll_loops {
        panic!("not implemented");
    }

    let raw_file = fs::read_to_string(args.file).expect("Failed reading file");
    let mut raw_file_tokens: Vec<BrainfuckToken> = Vec::new();

    for char in raw_file.into_bytes() {
        raw_file_tokens.push(match char {
            b'>' => BrainfuckToken::Next,
            b'<' => BrainfuckToken::Prev,
            b'+' => BrainfuckToken::Add,
            b'-' => BrainfuckToken::Sub,
            b'.' => BrainfuckToken::Out,
            b',' => BrainfuckToken::Input,
            b'[' => BrainfuckToken::LoopStart,
            b']' => BrainfuckToken::LoopEnd,
            _ => continue,
        });
    }

    // This is much easier to parse later
    let mut compressed_tokens: Vec<CompressedBrainfuckToken> = Vec::new();
    let mut index: usize = 0;
    while index < raw_file_tokens.len() {
        let currtoken = &raw_file_tokens[index];
        match raw_file_tokens[index] {
            BrainfuckToken::Out
            | BrainfuckToken::Input
            | BrainfuckToken::LoopStart
            | BrainfuckToken::LoopEnd => {
                compressed_tokens.push(CompressedBrainfuckToken {
                    token: currtoken.to_owned(),
                    num: 1,
                });
                index += 1;
                continue;
            }
            _ => (),
        }

        let mut numtokens: u64 = 0;
        let mut subindex: usize = index;
        while currtoken == &raw_file_tokens[subindex] {
            numtokens += 1;
            subindex += 1;

            if subindex == raw_file_tokens.len() {
                break;
            }
        }

        compressed_tokens.push(CompressedBrainfuckToken {
            token: currtoken.to_owned(),
            num: numtokens,
        });

        index += subindex - index;
    }

    // cleanup compressed tokens
    if !args.no_clean {
        let mut idx: usize = 0;
        while idx < compressed_tokens.len() - 1 {
            if !compressed_tokens[idx + 1].token.to_opposite().is_none()
                && compressed_tokens[idx].token
                    == compressed_tokens[idx + 1].token.to_opposite().unwrap()
            {
                if compressed_tokens[idx].num - compressed_tokens[idx + 1].num == 0 {
                    compressed_tokens.remove(idx);
                    compressed_tokens.remove(idx);
                } else if compressed_tokens[idx].num - compressed_tokens[idx + 1].num > 0 {
                    compressed_tokens[idx].num =
                        compressed_tokens[idx].num - compressed_tokens[idx + 1].num;
                    compressed_tokens.remove(idx + 1);
                } else if compressed_tokens[idx + 1].num - compressed_tokens[idx].num > 0 {
                    compressed_tokens[idx + 1].num =
                        compressed_tokens[idx + 1].num - compressed_tokens[idx].num;
                    compressed_tokens.remove(idx);
                }
                continue;
            }
            idx += 1;
        }
    }

    // check for bracket mismatch
    if !args.no_check {
        let mut index: usize = 0;
        while index < compressed_tokens.len() {
            if compressed_tokens[index].token != BrainfuckToken::LoopStart {
                index += 1;
                continue;
            }

            let mut balance = 0;
            let mut subindex: usize = index;
            while subindex < compressed_tokens.len() {
                match compressed_tokens[subindex].token {
                    BrainfuckToken::LoopStart => balance += 1,
                    BrainfuckToken::LoopEnd => balance -= 1,
                    _ => (),
                }

                if balance == 0 {
                    break;
                }

                subindex += 1;
            }

            if balance != 0 {
                panic!("Bracket mismatch!");
            }

            index += 1;
        }
    }

    // initialize QBE stack + pointer for BF program
    let mut mainfunc = qbe::Function::new(
        qbe::Linkage {
            exported: true,
            section: None,
            secflags: None,
        },
        "main".to_string(),
        vec![],
        Some(qbe::Type::Word),
    );

    let stack_name = String::from("stack");
    let stack_pointer_name = String::from("stackptr");

    let mut blocks: Vec<qbe::Block> = vec![];
    let mut blocksubidx = 1;

    let mut startblock = qbe::Block {
        label: format!("start.{}", blocksubidx),
        statements: vec![],
    };
    blocksubidx += 1;

    startblock.assign_instr(
        qbe::Value::Temporary(stack_name.to_owned()),
        qbe::Type::Long,
        qbe::Instr::Alloc4(30000),
    );

    startblock.assign_instr(
        qbe::Value::Temporary(stack_pointer_name.to_owned()),
        qbe::Type::Long,
        qbe::Instr::Alloc4(4),
    );
    blocks.push(startblock);

    let mut mainbody = qbe::Block {
        label: format!("body.{}", blocksubidx),
        statements: vec![],
    };
    blocksubidx += 1;

    mainbody.add_instr(qbe::Instr::Store(
        qbe::Type::Word,
        qbe::Value::Temporary(stack_pointer_name.to_owned()),
        qbe::Value::Const(0),
    ));
    blocks.push(mainbody);

    let mut loop_depth = 0;
    let mut while_loop_tags: Vec<WhileLoop> = vec![];
    const MUL_ROUNDING_VALUE: u64 = 1;
    let mut index: usize = 0;
    let mut varsubindex = 0;
    let mut currblockidx = 1; // start on body.1
    while index < compressed_tokens.len() {
        let currtoken = &compressed_tokens[index];
        let currblock: &mut qbe::Block = &mut blocks[currblockidx];
        match currtoken.token {
            BrainfuckToken::Next => {
                currblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Word,
                    qbe::Instr::Load(
                        qbe::Type::Word,
                        qbe::Value::Temporary(stack_pointer_name.to_owned()),
                    ),
                );
                varsubindex += 1;
                currblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Word,
                    qbe::Instr::Add(
                        qbe::Value::Temporary(format!(".{}", varsubindex - 1)),
                        qbe::Value::Const(currtoken.num),
                    ),
                );
                varsubindex += 1;
                currblock.add_instr(qbe::Instr::Store(
                    qbe::Type::Word,
                    qbe::Value::Temporary(stack_pointer_name.to_owned()),
                    qbe::Value::Temporary(format!(".{}", varsubindex - 1)),
                ));
            }
            BrainfuckToken::Prev => {
                currblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Word,
                    qbe::Instr::Load(
                        qbe::Type::Word,
                        qbe::Value::Temporary(stack_pointer_name.to_owned()),
                    ),
                );
                varsubindex += 1;
                currblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Word,
                    qbe::Instr::Sub(
                        qbe::Value::Temporary(format!(".{}", varsubindex - 1)),
                        qbe::Value::Const(currtoken.num),
                    ),
                );
                varsubindex += 1;
                currblock.add_instr(qbe::Instr::Store(
                    qbe::Type::Word,
                    qbe::Value::Temporary(stack_pointer_name.to_owned()),
                    qbe::Value::Temporary(format!(".{}", varsubindex - 1)),
                ));
            }
            BrainfuckToken::Add => {
                currblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Word,
                    qbe::Instr::Load(
                        qbe::Type::Word,
                        qbe::Value::Temporary(stack_pointer_name.to_owned()),
                    ),
                );
                varsubindex += 1;
                currblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Long,
                    qbe::Instr::Ext(
                        qbe::Type::SingleWord,
                        qbe::Value::Temporary(format!(".{}", varsubindex - 1)),
                    ),
                );
                varsubindex += 1;

                currblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Long,
                    qbe::Instr::Mul(
                        qbe::Value::Temporary(format!(".{}", varsubindex - 1)),
                        qbe::Value::Const(MUL_ROUNDING_VALUE),
                    ),
                );
                varsubindex += 1;

                let stackcopy = varsubindex;
                currblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Long,
                    qbe::Instr::Add(
                        qbe::Value::Temporary(stack_name.to_owned()),
                        qbe::Value::Temporary(format!(".{}", varsubindex - 1)),
                    ),
                );
                varsubindex += 1;

                currblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Word,
                    qbe::Instr::Load(
                        qbe::Type::SingleByte,
                        qbe::Value::Temporary(format!(".{}", varsubindex - 1)),
                    ),
                );
                varsubindex += 1;

                currblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Word,
                    qbe::Instr::Ext(
                        qbe::Type::SingleByte,
                        qbe::Value::Temporary(format!(".{}", varsubindex - 1)),
                    ),
                );
                varsubindex += 1;

                currblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Word,
                    qbe::Instr::Add(
                        qbe::Value::Temporary(format!(".{}", varsubindex - 1)),
                        qbe::Value::Const(currtoken.num),
                    ),
                );
                varsubindex += 1;
                currblock.add_instr(qbe::Instr::Store(
                    qbe::Type::Byte,
                    qbe::Value::Temporary(format!(".{}", stackcopy)),
                    qbe::Value::Temporary(format!(".{}", varsubindex - 1)),
                ));
            }
            BrainfuckToken::Sub => {
                currblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Word,
                    qbe::Instr::Load(
                        qbe::Type::Word,
                        qbe::Value::Temporary(stack_pointer_name.to_owned()),
                    ),
                );
                varsubindex += 1;
                currblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Long,
                    qbe::Instr::Ext(
                        qbe::Type::SingleWord,
                        qbe::Value::Temporary(format!(".{}", varsubindex - 1)),
                    ),
                );
                varsubindex += 1;

                currblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Long,
                    qbe::Instr::Mul(
                        qbe::Value::Temporary(format!(".{}", varsubindex - 1)),
                        qbe::Value::Const(MUL_ROUNDING_VALUE),
                    ),
                );
                varsubindex += 1;

                let stackcopy = varsubindex;
                currblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Long,
                    qbe::Instr::Add(
                        qbe::Value::Temporary(stack_name.to_owned()),
                        qbe::Value::Temporary(format!(".{}", varsubindex - 1)),
                    ),
                );
                varsubindex += 1;

                currblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Word,
                    qbe::Instr::Load(
                        qbe::Type::SingleByte,
                        qbe::Value::Temporary(format!(".{}", varsubindex - 1)),
                    ),
                );
                varsubindex += 1;

                currblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Word,
                    qbe::Instr::Ext(
                        qbe::Type::SingleByte,
                        qbe::Value::Temporary(format!(".{}", varsubindex - 1)),
                    ),
                );
                varsubindex += 1;

                currblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Word,
                    qbe::Instr::Sub(
                        qbe::Value::Temporary(format!(".{}", varsubindex - 1)),
                        qbe::Value::Const(currtoken.num),
                    ),
                );
                varsubindex += 1;
                currblock.add_instr(qbe::Instr::Store(
                    qbe::Type::Byte,
                    qbe::Value::Temporary(format!(".{}", stackcopy)),
                    qbe::Value::Temporary(format!(".{}", varsubindex - 1)),
                ));
            }
            BrainfuckToken::Input => {
                let getchar_place = varsubindex;
                currblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Word,
                    qbe::Instr::Call("getchar".to_owned(), vec![]),
                );
                varsubindex += 1;

                currblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Word,
                    qbe::Instr::Load(
                        qbe::Type::Word,
                        qbe::Value::Temporary(stack_pointer_name.to_owned()),
                    ),
                );
                varsubindex += 1;
                currblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Long,
                    qbe::Instr::Ext(
                        qbe::Type::SingleWord,
                        qbe::Value::Temporary(format!(".{}", varsubindex - 1)),
                    ),
                );
                varsubindex += 1;
                currblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Long,
                    qbe::Instr::Mul(
                        qbe::Value::Temporary(format!(".{}", varsubindex - 1)),
                        qbe::Value::Const(MUL_ROUNDING_VALUE),
                    ),
                );
                varsubindex += 1;

                let stackinst = varsubindex;
                currblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Long,
                    qbe::Instr::Add(
                        qbe::Value::Temporary(stack_name.to_owned()),
                        qbe::Value::Temporary(format!(".{}", varsubindex - 1)),
                    ),
                );
                varsubindex += 1;

                currblock.add_instr(qbe::Instr::Store(
                    qbe::Type::Byte,
                    qbe::Value::Temporary(format!(".{}", stackinst)),
                    qbe::Value::Temporary(format!(".{}", getchar_place)),
                ));
                varsubindex += 1;
            }
            BrainfuckToken::Out => {
                currblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Word,
                    qbe::Instr::Load(
                        qbe::Type::Word,
                        qbe::Value::Temporary(stack_pointer_name.to_owned()),
                    ),
                );
                varsubindex += 1;
                currblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Long,
                    qbe::Instr::Ext(
                        qbe::Type::SingleWord,
                        qbe::Value::Temporary(format!(".{}", varsubindex - 1)),
                    ),
                );
                varsubindex += 1;
                currblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Long,
                    qbe::Instr::Mul(
                        qbe::Value::Temporary(format!(".{}", varsubindex - 1)),
                        qbe::Value::Const(MUL_ROUNDING_VALUE),
                    ),
                );
                varsubindex += 1;
                currblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Long,
                    qbe::Instr::Add(
                        qbe::Value::Temporary(stack_name.to_owned()),
                        qbe::Value::Temporary(format!(".{}", varsubindex - 1)),
                    ),
                );
                varsubindex += 1;
                currblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Word,
                    qbe::Instr::Load(
                        qbe::Type::SingleByte,
                        qbe::Value::Temporary(format!(".{}", varsubindex - 1)),
                    ),
                );
                varsubindex += 1;
                currblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Word,
                    qbe::Instr::Ext(
                        qbe::Type::SingleByte,
                        qbe::Value::Temporary(format!(".{}", varsubindex - 1)),
                    ),
                );
                varsubindex += 1;

                for _ in 0..currtoken.num {
                    currblock.assign_instr(
                        qbe::Value::Temporary(format!(".{}", varsubindex)),
                        qbe::Type::Word,
                        qbe::Instr::Call(
                            String::from("putchar"),
                            vec![(
                                qbe::Type::Word,
                                qbe::Value::Temporary(format!(".{}", varsubindex - 1)),
                            )],
                        ),
                    );
                }
                varsubindex += 1;
            }
            BrainfuckToken::LoopStart => {
                let mut condblock = qbe::Block {
                    label: format!("while_cond.{}", blocksubidx),
                    statements: vec![],
                };
                blocksubidx += 1;

                condblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Word,
                    qbe::Instr::Load(
                        qbe::Type::Word,
                        qbe::Value::Temporary(stack_pointer_name.to_owned()),
                    ),
                );
                varsubindex += 1;
                condblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Long,
                    qbe::Instr::Ext(
                        qbe::Type::SingleWord,
                        qbe::Value::Temporary(format!(".{}", varsubindex - 1)),
                    ),
                );
                varsubindex += 1;
                condblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Long,
                    qbe::Instr::Mul(
                        qbe::Value::Temporary(format!(".{}", varsubindex - 1)),
                        qbe::Value::Const(MUL_ROUNDING_VALUE),
                    ),
                );
                varsubindex += 1;
                condblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Long,
                    qbe::Instr::Add(
                        qbe::Value::Temporary(stack_name.to_owned()),
                        qbe::Value::Temporary(format!(".{}", varsubindex - 1)),
                    ),
                );
                varsubindex += 1;
                condblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Word,
                    qbe::Instr::Load(
                        qbe::Type::SingleByte,
                        qbe::Value::Temporary(format!(".{}", varsubindex - 1)),
                    ),
                );
                varsubindex += 1;
                condblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Word,
                    qbe::Instr::Ext(
                        qbe::Type::SingleByte,
                        qbe::Value::Temporary(format!(".{}", varsubindex - 1)),
                    ),
                );
                varsubindex += 1;
                const POINTER_MUST_BE_X_AT_WHILE: u64 = 0;
                condblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Word,
                    qbe::Instr::Cmp(
                        qbe::Type::Word,
                        qbe::Cmp::Ne,
                        qbe::Value::Temporary(format!(".{}", varsubindex - 1)),
                        qbe::Value::Const(POINTER_MUST_BE_X_AT_WHILE),
                    ),
                );
                varsubindex += 1;
                condblock.add_instr(qbe::Instr::Jnz(
                    qbe::Value::Temporary(format!(".{}", varsubindex - 1)),
                    format!("while_body.{}", blocksubidx),
                    format!("while_join.{}", blocksubidx + 1), // body
                ));
                blocksubidx += 1; // register join
                while_loop_tags.push(WhileLoop {
                    join_num: blocksubidx - 2,
                });

                blocks.push(condblock);
                let newblock = qbe::Block {
                    label: format!("while_body.{}", blocksubidx - 1),
                    statements: vec![],
                };
                blocksubidx += 1;
                blocks.push(newblock);
                loop_depth += 1;
                currblockidx += 2;
            }
            BrainfuckToken::LoopEnd => {
                if loop_depth == 0 {
                    panic!("Unmatched loop end");
                }
                loop_depth -= 1;

                currblock.add_instr(qbe::Instr::Jmp(format!(
                    "while_cond.{}",
                    (while_loop_tags.last().unwrap().join_num)
                )));
                blocks.push(qbe::Block {
                    label: format!(
                        "while_join.{}",
                        while_loop_tags.last().unwrap().join_num + 2
                    ),
                    statements: vec![],
                });
                while_loop_tags.pop();
                currblockidx += 1;
            }
        }
        index += 1;
    }

    const RETURN_SUCCESS: u64 = 0;
    blocks
        .last_mut()
        .unwrap()
        .add_instr(qbe::Instr::Ret(Some(qbe::Value::Const(RETURN_SUCCESS))));
    mainfunc.blocks.append(&mut blocks);

    let mut bf_program = qbe::Module::new();
    bf_program.add_function(mainfunc);

    if !args.no_compile {
        let qbeproc = Command::new("qbe")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failure finding QBE binary");
        qbeproc
            .stdin
            .unwrap()
            .write_all(format!("{}", bf_program).as_bytes())
            .unwrap();
        let ccproc = Command::new("clang")
            .args(vec!["-OFast", "-x", "assembler", "-", "-v"])
            .stdin(Stdio::from(qbeproc.stdout.unwrap()))
            .spawn()
            .expect("Failure finding any C compiler through cc");

        println!(
            "{}",
            String::from_utf8(ccproc.wait_with_output().unwrap().stdout).unwrap()
        );
        return;
    }
    println!("{}", bf_program);
}
