//! Whole-project fixture driver. Compiled as a Rolldown example, not an Oxc microbenchmark.

use std::path::PathBuf;

use rolldown::{Bundler, BundlerOptions, InputItem, Platform, SourceMapType, TsConfig};
use rolldown_common::Output;

#[global_allocator]
static ALLOCATOR: mimalloc_safe::MiMalloc = mimalloc_safe::MiMalloc;

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
  let args: Vec<String> = std::env::args().collect();
  assert_eq!(args.len(), 6, "usage: project_build INPUT_ROOT PROJECT MINIFY MAP OUTPUT");
  let root = PathBuf::from(&args[1]);
  let project = args[2].as_str();
  let minify = args[3] == "true";
  let maps = args[4] == "true";
  let cwd = root.join(match project {
    "three" => "apps/three10x/copy1",
    "rome" => "apps/rome",
    _ => panic!("unknown project: {project}"),
  });
  let is_rome = project == "rome";
  let mut bundler = Bundler::new(BundlerOptions {
    input: Some(vec![InputItem {
      name: Some("main".to_owned()),
      import: if is_rome { "./src/entry.ts" } else { "./Three.js" }.to_owned(),
    }]),
    platform: Some(if is_rome { Platform::Node } else { Platform::Browser }),
    sourcemap: maps.then_some(SourceMapType::File),
    minify: Some(minify.into()),
    shim_missing_exports: Some(is_rome),
    tsconfig: is_rome.then(|| TsConfig::Manual(cwd.join("src/tsconfig.json"))),
    dir: Some(args[5].clone()),
    cwd: Some(cwd),
    ..Default::default()
  })
  .expect("create bundler");
  let output = bundler.write().await.expect("build and write project");
  let mut modules = 0;
  let mut chunks = 0;
  let mut mapped_chunks = 0;
  for asset in &output.assets {
    if let Output::Chunk(chunk) = asset {
      modules += chunk.module_ids.len();
      chunks += 1;
      mapped_chunks += usize::from(chunk.map.is_some());
    }
  }
  assert!(modules > 100, "fixture must exercise a real module graph");
  assert_eq!(mapped_chunks, if maps { chunks } else { 0 });
  println!(
    "CHECK {project} minify={minify} maps={maps} modules={modules} chunks={chunks} warnings={}",
    output.warnings.len()
  );
  drop(output);
  bundler.close().await.expect("close bundle");
}
