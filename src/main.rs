use clap::Parser;
use qbe::Block;
use std::fs;

#[derive(Parser)]
struct Cli {
    file: String,
    #[clap(short, long, default_value_t = false)]
    no_check: bool,
    #[clap(short, long, default_value_t = false)]
    unroll_loops: bool,
}

#[derive(Debug, PartialEq, Eq, Clone)]
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

#[derive(Debug, PartialEq, Eq, Clone)]
struct CompressedBrainfuckToken {
    token: BrainfuckToken,
    num: u64,
}

fn main() {
    let args = Cli::parse();
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

    // check for bracket mismatch
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
            subindex += 1;
        }

        if balance != 0 {
            panic!("Bracket mismatch!");
        }

        index += 1;
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

    let mut blocknumindex = 1;
    let startblock = mainfunc.add_block(format!("start.{}", blocknumindex));
    blocknumindex += 1;

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

    const MUL_ROUNDING_VALUE: u64 = 1;

    // transpile to BF program to QBE-IR
    let mut mainbody = mainfunc.add_block(format!("body.{}", blocknumindex));

    mainbody.add_instr(qbe::Instr::Store(
        qbe::Type::Word,
        qbe::Value::Temporary(stack_pointer_name.to_owned()),
        qbe::Value::Const(0),
    ));

    let mut loop_depth = 0;
    let mut loop_start_blocks: Vec<qbe::Block> = Vec::new();

    // let mut loop_end_blocks = Vec::new();

    let mut index: usize = 0;
    let mut varsubindex = 0;
    let mut currblock: &mut Block = &mut mainbody;
    while index < compressed_tokens.len() {
        let currtoken = &compressed_tokens[index];
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
                        qbe::Instr::Call("getchar".to_owned(), vec![]),
                    );
                }
                varsubindex += 1;

                currblock.assign_instr(
                    qbe::Value::Temporary(format!(".{}", varsubindex)),
                    qbe::Type::Word,
                    qbe::Instr::Add(
                        qbe::Value::Temporary(format!(".{}", varsubindex - 2)),
                        qbe::Value::Temporary(format!(".{}", varsubindex - 1)),
                    ),
                );
                varsubindex += 1;

                currblock.add_instr(qbe::Instr::Store(
                    qbe::Type::Byte,
                    qbe::Value::Temporary(format!(".{}", stackinst)),
                    qbe::Value::Temporary(format!(".{}", varsubindex - 1)),
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
                    label: format!("while_cond.{}", blocknumindex),
                    statements: vec![],
                };
                blocknumindex += 1;

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
                        qbe::Value::Temporary(format!(".{}", varsubindex)),
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
                    format!("while_body.{}", blocknumindex + 1),
                    format!("while_join.{}", 1),
                ));
                blocknumindex += 1;

                mainfunc.blocks.push(condblock);
                let newblock = qbe::Block {
                    label: format!("while_body.{}", blocknumindex),
                    statements: vec![],
                };
                loop_start_blocks.push(newblock);
                loop_depth += 1;
                currblock = &mut loop_start_blocks.last_mut().unwrap();
            }
            BrainfuckToken::LoopEnd => {
                if loop_depth == 0 {
                    panic!("Unmatched loop end");
                }
                loop_depth -= 1;

                let lastblock = loop_start_blocks.pop();
                // currblock = loop_start_blocks.last_mut().unwrap();
            }
        }
        index += 1;
    }

    const RETURN_SUCCESS: u64 = 0;
    mainfunc
        .add_block("end")
        .add_instr(qbe::Instr::Ret(Some(qbe::Value::Const(RETURN_SUCCESS))));

    let mut bf_program = qbe::Module::new();
    bf_program.add_function(mainfunc);
    println!("{}", bf_program);
}
