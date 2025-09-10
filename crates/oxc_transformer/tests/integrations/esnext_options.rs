use oxc_transformer::{ESTarget, TransformOptions};

#[test]
fn test_esnext_explicit_resource_management() {
    // Test that esnext target disables explicit resource management
    let options = TransformOptions::from(ESTarget::ESNext);
    assert!(!options.env.esnext.explicit_resource_management);
    
    // Test that other targets enable explicit resource management
    let options = TransformOptions::from(ESTarget::ES2022);
    assert!(options.env.esnext.explicit_resource_management);
    
    // Test from target string
    let options = TransformOptions::from_target("esnext").unwrap();
    assert!(!options.env.esnext.explicit_resource_management);
    
    // Test from target list with esnext
    let options = TransformOptions::from_target_list(&["esnext", "chrome100"]).unwrap();
    assert!(!options.env.esnext.explicit_resource_management);
    
    // Test from target list without esnext
    let options = TransformOptions::from_target_list(&["es2022", "chrome100"]).unwrap();
    assert!(options.env.esnext.explicit_resource_management);
}