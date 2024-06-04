use criterion::{criterion_group, criterion_main, Criterion};
use prop::strategy::ValueTree;
use proptest::prelude::*;
mod test_utils;


fn benchmark_process_data(c: &mut Criterion) {
    // 定义一个策略，用于生成各种长度的字符串
    let strategy = test_utils::arb_json;

    // 生成一些测试用例
    let mut test_cases = vec![];
    for _ in 0..100 {
        let case: String = strategy().new_tree(&mut proptest::test_runner::TestRunner::default())
                                  .unwrap()
                                  .current()
                                  .to_string();
        test_cases.push(case);
    }

    // 基准测试
    c.bench_function("process_data", |b| {
        b.iter_batched_ref(
            || test_cases.clone(),
            |cases| {
                for case in cases {
                    partial_json_parser_rs::Parser::parser(case).unwrap();
                }
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, benchmark_process_data);
criterion_main!(benches);