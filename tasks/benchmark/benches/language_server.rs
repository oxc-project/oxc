use oxc_benchmark::{BenchmarkId, Criterion, criterion_group, criterion_main};
use oxc_language_server::{
    Backend, ServerLinterBuilder, TestServer, WORKSPACE, did_open, initialize_request,
    initialized_notification, server_info, shutdown_request,
};
use oxc_tasks_common::TestFiles;

fn bench_linter(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("language_server");

    for file in TestFiles::minimal().files() {
        let id = BenchmarkId::from_parameter(format!("{}-linter", file.file_name));
        let source_text = &file.source_text;
        let uri = format!("file:///{WORKSPACE}{}", file.file_name);

        group.bench_function(id, |b| {
            b.to_async(tokio::runtime::Runtime::new().unwrap()).iter(|| async {
                let mut server = TestServer::new(|client| {
                    Backend::new(client, server_info(), vec![Box::new(ServerLinterBuilder)])
                });
                // Send initialize request
                server.send_request(initialize_request(false, false, false, None)).await;
                let _ = server.recv_response().await;

                // Send initialized notification
                server.send_request(initialized_notification()).await;

                // Send didOpen notification, expecting the linter to run
                server.send_request(did_open(&uri, source_text)).await;

                // Shutdown the server
                server.send_request(shutdown_request(2)).await;
                let _ = server.recv_response().await;
            });
        });
    }
    group.finish();
}

criterion_group!(language_server, bench_linter);
criterion_main!(language_server);
