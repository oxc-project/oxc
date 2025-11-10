// Baseline lexer benchmarks with embedded test cases
// This avoids dependency on external file downloads

use oxc_allocator::Allocator;
use oxc_benchmark::{BenchmarkId, Criterion, criterion_group, criterion_main};
use oxc_parser::lexer::{Kind, Lexer};
use oxc_span::SourceType;

// Embedded realistic JavaScript samples
const SMALL_JS: &str = r#"
function fibonacci(n) {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

const result = fibonacci(10);
console.log(result);
"#;

const MEDIUM_JS: &str = r#"
import { useState, useEffect } from 'react';

export function Counter() {
    const [count, setCount] = useState(0);
    const [loading, setLoading] = useState(false);

    useEffect(() => {
        const timer = setTimeout(() => {
            setLoading(false);
        }, 1000);
        return () => clearTimeout(timer);
    }, []);

    const increment = () => setCount(c => c + 1);
    const decrement = () => setCount(c => c - 1);

    return (
        <div className="counter">
            <button onClick={decrement}>-</button>
            <span>{count}</span>
            <button onClick={increment}>+</button>
        </div>
    );
}
"#;

const LARGE_JS: &str = r#"
// Complex class with various JavaScript features
class DataProcessor {
    #privateField = null;
    static version = '1.0.0';

    constructor(config = {}) {
        this.config = { timeout: 5000, retries: 3, ...config };
        this.#privateField = new WeakMap();
        this.cache = new Map();
    }

    async processData(data) {
        const startTime = performance.now();

        try {
            const validated = await this.validate(data);
            const transformed = this.transform(validated);
            const result = await this.save(transformed);

            return {
                success: true,
                result,
                duration: performance.now() - startTime
            };
        } catch (error) {
            console.error('Processing failed:', error);
            return { success: false, error: error.message };
        }
    }

    async validate(data) {
        if (!data || typeof data !== 'object') {
            throw new Error('Invalid data format');
        }

        const required = ['id', 'name', 'value'];
        for (const field of required) {
            if (!(field in data)) {
                throw new Error(`Missing required field: ${field}`);
            }
        }

        return data;
    }

    transform(data) {
        return {
            ...data,
            id: String(data.id).padStart(8, '0'),
            name: data.name.trim().toLowerCase(),
            value: Number(data.value),
            timestamp: new Date().toISOString(),
            checksum: this.#calculateChecksum(data)
        };
    }

    async save(data) {
        const key = data.id;

        if (this.cache.has(key)) {
            return this.cache.get(key);
        }

        const saved = await this.#performSave(data);
        this.cache.set(key, saved);

        return saved;
    }

    #calculateChecksum(data) {
        const str = JSON.stringify(data);
        let hash = 0;
        for (let i = 0; i < str.length; i++) {
            const char = str.charCodeAt(i);
            hash = ((hash << 5) - hash) + char;
            hash = hash & hash;
        }
        return hash.toString(36);
    }

    async #performSave(data) {
        return new Promise((resolve, reject) => {
            setTimeout(() => {
                if (Math.random() > 0.1) {
                    resolve({ ...data, saved: true });
                } else {
                    reject(new Error('Save failed'));
                }
            }, 100);
        });
    }

    static create(config) {
        return new DataProcessor(config);
    }

    get status() {
        return {
            version: DataProcessor.version,
            cacheSize: this.cache.size,
            config: this.config
        };
    }

    *[Symbol.iterator]() {
        yield* this.cache.entries();
    }
}

export default DataProcessor;

// Usage examples
const processor = DataProcessor.create({ timeout: 3000 });

const testData = [
    { id: 1, name: '  Test Item  ', value: '42' },
    { id: 2, name: 'Another Item', value: '99' },
    { id: 3, name: 'Final Item', value: '0' }
];

Promise.all(testData.map(data => processor.processData(data)))
    .then(results => console.log('All processed:', results))
    .catch(error => console.error('Batch failed:', error));
"#;

const TYPESCRIPT_SAMPLE: &str = r#"
interface User {
    id: number;
    name: string;
    email?: string;
    roles: readonly string[];
}

type UserRole = 'admin' | 'user' | 'guest';

class UserManager<T extends User> {
    private users: Map<number, T> = new Map();

    constructor(private readonly config: { maxUsers: number }) {}

    add(user: T): void {
        if (this.users.size >= this.config.maxUsers) {
            throw new Error('Maximum users reached');
        }
        this.users.set(user.id, user);
    }

    get(id: number): T | undefined {
        return this.users.get(id);
    }

    filter(predicate: (user: T) => boolean): T[] {
        return Array.from(this.users.values()).filter(predicate);
    }
}

const manager = new UserManager<User>({ maxUsers: 100 });
manager.add({ id: 1, name: 'Alice', roles: ['admin'] });
"#;

fn bench_lexer_baseline(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("lexer_baseline");

    let test_cases = [
        ("small_js", SMALL_JS, SourceType::default()),
        ("medium_js", MEDIUM_JS, SourceType::default().with_module(true).with_jsx(true)),
        ("large_js", LARGE_JS, SourceType::default().with_module(true)),
        ("typescript", TYPESCRIPT_SAMPLE, SourceType::default().with_typescript(true)),
    ];

    for (name, source, source_type) in test_cases {
        let id = BenchmarkId::from_parameter(name);
        group.bench_function(id, |b| {
            let mut allocator = Allocator::default();
            b.iter(|| {
                lex_all(&allocator, source, source_type);
                allocator.reset();
            });
        });
    }
    group.finish();
}

fn bench_token_rate(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("token_rate");
    group.throughput(oxc_benchmark::criterion::Throughput::Elements(1));

    // Measure tokens per second
    let source = LARGE_JS;
    let source_type = SourceType::default().with_module(true);

    group.bench_function("tokens_per_second", |b| {
        let mut allocator = Allocator::default();
        b.iter(|| {
            let token_count = lex_and_count(&allocator, source, source_type);
            allocator.reset();
            token_count
        });
    });

    group.finish();
}

fn bench_bytes_per_second(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("throughput");

    for (name, source, source_type) in [
        ("small", SMALL_JS, SourceType::default()),
        ("medium", MEDIUM_JS, SourceType::default().with_module(true)),
        ("large", LARGE_JS, SourceType::default().with_module(true)),
    ] {
        let bytes = source.len() as u64;
        group.throughput(oxc_benchmark::criterion::Throughput::Bytes(bytes));

        let id = BenchmarkId::from_parameter(name);
        group.bench_function(id, |b| {
            let mut allocator = Allocator::default();
            b.iter(|| {
                lex_all(&allocator, source, source_type);
                allocator.reset();
            });
        });
    }

    group.finish();
}

// Helper function to lex entire source
#[inline(always)]
fn lex_all<'a>(
    allocator: &'a Allocator,
    source_text: &'a str,
    source_type: SourceType,
) -> Lexer<'a> {
    let mut lexer = Lexer::new_for_benchmarks(allocator, source_text, source_type);
    if lexer.first_token().kind() != Kind::Eof {
        while lexer.next_token_for_benchmarks().kind() != Kind::Eof {}
    }
    lexer
}

// Helper to count tokens
#[inline(always)]
fn lex_and_count(
    allocator: &Allocator,
    source_text: &str,
    source_type: SourceType,
) -> usize {
    let mut lexer = Lexer::new_for_benchmarks(allocator, source_text, source_type);
    let mut count = 0;

    if lexer.first_token().kind() != Kind::Eof {
        count = 1;
        while lexer.next_token_for_benchmarks().kind() != Kind::Eof {
            count += 1;
        }
    }

    count
}

criterion_group!(
    lexer_baseline,
    bench_lexer_baseline,
    bench_token_rate,
    bench_bytes_per_second,
);
criterion_main!(lexer_baseline);
