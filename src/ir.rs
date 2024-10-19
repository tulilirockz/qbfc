pub struct QBEIr<'a> {
    pub program: qbe::Module<'a>,
    main_func: qbe::Function<'a>,
}

impl<'a> QBEIr<'a> {
    pub fn new() -> Self {
        let mainfunc = qbe::Function::new(
            qbe::Linkage {
                exported: true,
                section: None,
                secflags: None,
            },
            "main".to_string(),
            vec![],
            Some(qbe::Type::Word),
        );

        QBEIr {
            program: qbe::Module::new(),
            main_func: mainfunc,
        }
    }
    pub fn close_prog(self: &mut Self) {
        self.program.add_function(self.main_func.clone());
    }
}
