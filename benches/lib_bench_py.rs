use criterion::{criterion_group, criterion_main, Criterion};
use pyo3::{
    types::{PyAnyMethods, PyModule},
    Python,
};
mod test_utils;

fn benchmark_process_data_py(c: &mut Criterion) {
    // 初始化 Python 解释器
    pyo3::prepare_freethreaded_python();

    Python::with_gil(|py| {
        // 导入 partial_json_parser 模块
        let partial_json_parser = PyModule::import_bound(py, "partial_json_parser").unwrap();

        let test_cases = test_utils::gen_test_cases(100);

        // 基准测试
        c.bench_function("process_data_py", |b| {
            b.iter_batched_ref(
                || test_cases.clone(),
                |cases| {
                    for case in cases {
                        // 获取 ensure_json 函数
                        let ensure_json = partial_json_parser.getattr("ensure_json").unwrap();

                        // 调用 ensure_json 函数并提取结果
                        #[allow(unused)]
                        let res = ensure_json
                            .call1((case as &String,))
                            .unwrap()
                            .extract::<String>();
                    }
                },
                criterion::BatchSize::SmallInput,
            )
        });
    });
}

criterion_group!(benches1, benchmark_process_data_py);
criterion_main!(benches1);
