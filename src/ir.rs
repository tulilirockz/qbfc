use crate::{BrainfuckToken, CompressedBrainfuckToken};

pub struct QBEIr<'a> {
    pub program: qbe::Module<'a>,
    main_func: qbe::Function<'a>,
    blocks: BlocksTracker<'a>,
}

struct BlocksTracker<'a> {
    blocks: Vec<qbe::Block<'a>>,
    idx: usize,
}

impl<'a> BlocksTracker<'a> {
    fn new() -> Self {
        return BlocksTracker {
            blocks: vec![],
            idx: 1,
        };
    }

    fn add_block(self: &mut Self, block: qbe::Block<'a>) {
        self.blocks.push(block);
        self.idx += 1;
    }
}

impl<'a> QBEIr<'a> {
    pub fn new() -> Self {
        QBEIr {
            program: qbe::Module::new(),
            main_func: qbe::Function::new(
                qbe::Linkage {
                    exported: true,
                    section: None,
                    secflags: None,
                },
                "main".to_string(),
                vec![],
                Some(qbe::Type::Word),
            ),
            blocks: BlocksTracker::new(),
        }
    }

    pub fn init_body(self: &mut Self) -> &mut Self {
        let stack_name = String::from("stack");
        let stack_pointer_name = String::from("stackptr");
        let mut startblock = qbe::Block {
            label: format!("start.{}", self.blocks.idx),
            statements: vec![],
        };

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
        self.blocks.add_block(startblock);

        let mut mainbody = qbe::Block {
            label: format!("body.{}", self.blocks.idx),
            statements: vec![],
        };

        mainbody.add_instr(qbe::Instr::Store(
            qbe::Type::Word,
            qbe::Value::Temporary(stack_pointer_name.to_owned()),
            qbe::Value::Const(0),
        ));
        self.blocks.add_block(mainbody);
        self
    }

    pub fn close_prog(self: &mut Self) {
        const RETURN_SUCCESS: u64 = 0;
        self.blocks
            .blocks
            .last_mut()
            .unwrap()
            .add_instr(qbe::Instr::Ret(Some(qbe::Value::Const(RETURN_SUCCESS))));
        self.main_func.blocks.append(&mut self.blocks.blocks);

        self.program.add_function(self.main_func.clone());
    }

    pub fn token_array_to_qbe_ir(
        self: &mut Self,
        compressed_tokens: Vec<CompressedBrainfuckToken>,
    ) -> &mut Self {
        let stack_name = String::from("stack");
        let stack_pointer_name = String::from("stackptr");

        let mut loop_depth = 0;
        let mut while_loop_tags: Vec<usize> = vec![];
        const MUL_ROUNDING_VALUE: u64 = 1;
        let mut index: usize = 0;
        let mut varsubindex = 0;
        let mut currblockidx = 1; // start on body.1
        while index < compressed_tokens.len() {
            let currtoken = &compressed_tokens[index];
            let currblock: &mut qbe::Block = &mut self.blocks.blocks[currblockidx];
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
                        label: format!("while_cond.{}", self.blocks.idx),
                        statements: vec![],
                    };
                    self.blocks.idx += 1;

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
                        format!("while_body.{}", self.blocks.idx),
                        format!("while_join.{}", self.blocks.idx + 1), // body
                    ));
                    self.blocks.idx += 1; // register join
                    while_loop_tags.push(self.blocks.idx - 2);

                    self.blocks.blocks.push(condblock);
                    let newblock = qbe::Block {
                        label: format!("while_body.{}", self.blocks.idx - 1),
                        statements: vec![],
                    };
                    self.blocks.idx += 1;
                    self.blocks.blocks.push(newblock);
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
                        (while_loop_tags.last().unwrap())
                    )));
                    self.blocks.blocks.push(qbe::Block {
                        label: format!("while_join.{}", while_loop_tags.last().unwrap() + 2),
                        statements: vec![],
                    });
                    while_loop_tags.pop();
                    currblockidx += 1;
                }
                BrainfuckToken::Invalid => (),
            }
            index += 1;
        }
        self
    }
}
