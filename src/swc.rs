use std::fs;
use std::path::Path;
use swc_common::{sync::Lrc, FileName, Mark, SourceMap};
use swc_ecma_ast::Module;
use swc_ecma_codegen::{text_writer::JsWriter, Config, Emitter};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};
use swc_ecma_transforms::typescript::strip;
use swc_ecma_visit::FoldWith;

fn compile_typescript_to_javascript(ts_code: &str) -> String {
    let cm = Lrc::new(SourceMap::default());
    let fm = cm.new_source_file(FileName::Anon.into(), ts_code.to_string());

    let lexer = Lexer::new(
        Syntax::Typescript(Default::default()),
        swc_ecma_ast::EsVersion::Es2020,
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);
    let mut module: Module = parser.parse_module().unwrap();

    let unresolved_mark = Mark::new();
    let top_level_mark = Mark::new();

    // Use fold_with instead of visit_mut_with
    module = module.fold_with(&mut strip(unresolved_mark, top_level_mark));

    let mut buf = Vec::new();
    {
        let writer = Box::new(JsWriter::new(Lrc::clone(&cm), "\n", &mut buf, None));
        let mut emitter = Emitter {
            cfg: Config::default(),
            cm: Lrc::clone(&cm),
            comments: None,
            wr: writer,
        };
        emitter.emit_module(&module).unwrap();
    }

    String::from_utf8(buf).unwrap()
}

pub fn compile_folder(input_path: &Path, output_path: &Path) {
    if input_path.is_dir() {
        fs::create_dir_all(output_path).unwrap();

        for entry in fs::read_dir(input_path).unwrap() {
            let entry = entry.unwrap();
            let entry_path = entry.path();
            let output_entry_path = output_path.join(entry_path.file_name().unwrap());

            if entry_path.is_dir() {
                compile_folder(&entry_path, &output_entry_path);
            } else if entry_path.extension().and_then(|s| s.to_str()) == Some("ts") {
                let ts_code = fs::read_to_string(&entry_path).unwrap();
                let js_code = compile_typescript_to_javascript(&ts_code);
                fs::write(output_entry_path.with_extension("js"), js_code).unwrap();
            }
        }
    }
}
