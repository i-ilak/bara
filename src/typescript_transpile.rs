use swc::{config::IsModule, Compiler, PrintArgs};
use swc_common::{errors::Handler, source_map::SourceMap, sync::Lrc, Mark, GLOBALS};
use swc_ecma_ast::EsVersion;
use swc_ecma_parser::Syntax;
use swc_ecma_transforms_typescript::strip;
use swc_ecma_visit::FoldWith;
use walkdir::WalkDir;
use std::sync::Arc;
use std::path::Path;

use crate::config::ConfigFile;
use crate::util::write_file;

/// Transforms typescript to javascript. Returns tuple (js string, source map)
fn ts_to_js(filename: &str, ts_code: &str) -> (String, String) {
    let cm = Lrc::new(SourceMap::new(swc_common::FilePathMapping::empty()));

    let compiler = Compiler::new(cm.clone());

    let source = cm.new_source_file(
        Arc::new(swc_common::FileName::Custom(filename.into())),
        ts_code.to_string(),
    );

    let handler = Handler::with_emitter_writer(Box::new(std::io::stderr()), Some(compiler.cm.clone()));

    return GLOBALS.set(&Default::default(), || {
        let program = compiler
            .parse_js(
                source,
                &handler,
                EsVersion::Es5,
                Syntax::Typescript(Default::default()),
                IsModule::Bool(false),
                Some(compiler.comments()),
            )
            .expect("parse_js failed");

            let unresolved_mark = Mark::new();
            let top_level_mark = Mark::new();
            use swc_ecma_transforms_base::chain;
            let program = program.fold_with(&mut chain!(strip(unresolved_mark, top_level_mark)));

        let ret = compiler
            .print(
                &program, // ast to print
                PrintArgs::default(),
            )
            .expect("print failed");

        return (ret.code, ret.map.expect("no source map"));
    });
}

pub async fn transpile_typescript_files(
    config: &ConfigFile,
    output_folder: &Path,
)
{
    let input_dir = config.root.join("scripts");
    for entry in WalkDir::new(input_dir).into_iter().filter_map(|e| e.ok()) {
        let ts_code = std::fs::read_to_string(entry.path()).expect("Could not read typescript file!");
        let (js, _) = ts_to_js(
            entry.file_name()
                .to_str()
                .expect("Could not convert path to string"), 
            &ts_code);
        
        write_file(&output_folder.join(entry.file_name()), js);
    }
}