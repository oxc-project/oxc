use oxc_estree::{CompactTSSerializer, ESTree, JsonSafeString, Serializer, StructSerializer};
use oxc_span::Span;

struct TestSpan {
    span: Span,
    use_loc: bool,
    start_line: u32,
    start_column: u32,
    end_line: u32,
    end_column: u32,
}

impl ESTree for TestSpan {
    fn serialize<S: Serializer>(&self, serializer: S) {
        let mut state = serializer.serialize_struct();
        state.serialize_field("type", &JsonSafeString("TestNode"));
        
        if self.use_loc && serializer.loc() {
            // Use serialize_span_with_loc with actual location info
            state.serialize_span_with_loc(
                self.span, 
                (self.start_line, self.start_column), 
                (self.end_line, self.end_column)
            );
        } else {
            // Use regular serialize_span (will show placeholder loc)
            state.serialize_span(self.span);
        }
        
        state.end();
    }
}

fn main() {
    println!("=== Testing serialize_span vs serialize_span_with_loc ===\n");
    
    let span = Span::new(10, 20);
    
    // Test 1: Regular serialize_span with loc=true (should show (0,0) placeholder)
    println("1. Regular serialize_span with loc=true:");
    let test1 = TestSpan {
        span,
        use_loc: false,
        start_line: 0,
        start_column: 0,
        end_line: 0,
        end_column: 0,
    };
    let mut serializer1 = CompactTSSerializer::with_capacity_and_loc(1024, false, true);
    test1.serialize(&mut serializer1);
    println!("  Result: {}\n", serializer1.into_string());
    
    // Test 2: serialize_span_with_loc with real location info
    println!("2. serialize_span_with_loc with real location info:");
    let test2 = TestSpan {
        span,
        use_loc: true,
        start_line: 1,  // line 2 (0-based)
        start_column: 5,
        end_line: 2,    // line 3 (0-based) 
        end_column: 15,
    };
    let mut serializer2 = CompactTSSerializer::with_capacity_and_loc(1024, false, true);
    test2.serialize(&mut serializer2);
    println!("  Result: {}\n", serializer2.into_string());
    
    println!("✅ If loc fields show different values, serialize_span_with_loc is working!");
}