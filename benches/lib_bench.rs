use criterion::{criterion_group, criterion_main, Criterion};
mod test_utils;

fn benchmark_process_data(c: &mut Criterion) {
    let test_cases = test_utils::gen_test_cases(100);

    // 基准测试
    c.bench_function("process_data", |b| {
        b.iter_batched_ref(
            || test_cases.clone(),
            |cases| {
                for case in cases {
                    #[allow(unused)]
                    let res = partial_json_parser_rs::Parser::parser(case);
                }
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, benchmark_process_data);
criterion_main!(benches);
