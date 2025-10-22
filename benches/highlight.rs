use criterion::{Criterion, criterion_group, criterion_main};
use gitu::syntax_parser;
use similar;
use std::path::Path;
use unicode_segmentation::UnicodeSegmentation;

fn benchmark_syntax_parsing(c: &mut Criterion) {
    let large_rust_code = r#"
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct ComplexStruct {
    field1: String,
    field2: Vec<i32>,
    field3: HashMap<String, Arc<String>>,
    field4: Option<Box<ComplexStruct>>,
}

impl ComplexStruct {
    pub fn new() -> Self {
        Self {
            field1: "Hello World".to_string(),
            field2: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            field3: HashMap::new(),
            field4: None,
        }
    }

    pub fn process(&mut self, data: &[u8]) -> Result<(), String> {
        if data.is_empty() {
            return Err("Empty data".to_string());
        }

        for (i, &byte) in data.iter().enumerate() {
            self.field2.push(byte as i32);
            self.field3.insert(format!("key_{}", i), Arc::new(format!("value_{}", byte)));
        }

        self.field4 = Some(Box::new(Self::new()));
        Ok(())
    }

    pub fn complex_computation(&self) -> i64 {
        let mut sum = 0i64;
        for &val in &self.field2 {
            sum += val as i64 * val as i64;
        }
        for (_, arc_val) in &self.field3 {
            sum += arc_val.len() as i64;
        }
        if let Some(ref nested) = self.field4 {
            sum += nested.complex_computation();
        }
        sum
    }
}

fn async_process(data: Vec<u8>) -> impl std::future::Future<Output = Result<ComplexStruct, String>> {
    async move {
        let mut instance = ComplexStruct::new();
        instance.process(&data)?;
        Ok(instance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complex_struct() {
        let mut instance = ComplexStruct::new();
        let data = vec![1, 2, 3, 4, 5];
        instance.process(&data).unwrap();
        assert_eq!(instance.field2.len(), 15);
        assert_eq!(instance.complex_computation(), 335); // Pre-calculated value
    }

    #[tokio::test]
    async fn test_async_process() {
        let data = vec![10, 20, 30];
        let result = async_process(data).await.unwrap();
        assert_eq!(result.field2.len(), 13);
    }
}
"#.repeat(10); // Make it larger

    c.bench_function("syntax_parsing_large_rust", |b| {
        b.iter(|| {
            let _ = syntax_parser::parse(Path::new("test.rs"), &large_rust_code);
        })
    });
}

fn benchmark_diff_computation(c: &mut Criterion) {
    // Create sample diff content with changes
    let old_content = "line1\nline2\nline3\nline4\nline5\n";
    let new_content = "line1\nmodified_line2\nline3\nnew_line\nline4\nline5\n";

    let old_tokens: Vec<&str> = old_content.unicode_words().collect();
    let new_tokens: Vec<&str> = new_content.unicode_words().collect();

    c.bench_function("diff_computation_patience", |b| {
        b.iter(|| {
            let _ = similar::capture_diff_slices(
                similar::Algorithm::Patience,
                &old_tokens,
                &new_tokens,
            );
        })
    });
}

criterion_group! {
    name = highlight_benches;
    config = Criterion::default();
    targets = benchmark_syntax_parsing, benchmark_diff_computation
}
criterion_main!(highlight_benches);
