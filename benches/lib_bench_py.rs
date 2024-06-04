use criterion::{criterion_group, criterion_main, Criterion};
use prop::strategy::ValueTree;
use proptest::prelude::*;
use pyo3::{types::{PyAnyMethods, PyModule}, PyResult, Python};
mod test_utils;

/// 封装的函数，用于调用 Python 的 ensure_json 并返回补全后的 JSON 字符串
fn complete_json(partial_json: &str) -> PyResult<String> {
    Python::with_gil(|py| {
        // 导入 partial_json_parser 模块
        let partial_json_parser = PyModule::import_bound(py, "partial_json_parser")?;
        
        // 获取 ensure_json 函数
        let ensure_json = partial_json_parser.getattr("ensure_json")?;
        
        // 调用 ensure_json 函数并提取结果
        let result: String = ensure_json.call1((partial_json,))?.extract()?;
        
        Ok(result)
    })
}


fn benchmark_process_data_py(c: &mut Criterion) {
    // 定义一个策略，用于生成各种长度的字符串
    let strategy = test_utils::arb_json;

    // 初始化 Python 解释器
    pyo3::prepare_freethreaded_python();

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
                    complete_json(case).unwrap();
                }
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, benchmark_process_data_py);
criterion_main!(benches);