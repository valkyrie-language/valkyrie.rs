use nyar_backend_jvm::{JvmClassFile, JvmJarPackage, JvmMethodDescriptor, JvmTypeDescriptor};

#[test]
fn round_trips_jar_archive() {
    let mut package = JvmJarPackage::new("demo.jar");
    package.main_class = Some("demo.Main".to_string());
    package.push_entry("demo/data.txt", b"hello".to_vec());

    let bytes = package.to_bytes().unwrap();
    let decoded = JvmJarPackage::from_bytes("demo.jar", &bytes).unwrap();

    assert_eq!(decoded.main_class.as_deref(), Some("demo.Main"));
    assert_eq!(decoded.entries.len(), 1);
    assert_eq!(decoded.entries[0].path, "demo/data.txt");
}

#[test]
fn embeds_and_reads_class_files() {
    let mut class_file = JvmClassFile::new("demo/Main");
    class_file.push_method(
        "main",
        JvmMethodDescriptor::new(
            vec![JvmTypeDescriptor::array(JvmTypeDescriptor::Object("java/lang/String".to_string()))],
            JvmTypeDescriptor::Void,
        ),
    );

    let mut package = JvmJarPackage::new("demo.jar");
    package.push_class(&class_file).unwrap();

    let decoded = package.read_class("demo/Main").unwrap().unwrap();
    assert_eq!(decoded.internal_name, "demo/Main");
    assert_eq!(decoded.methods[0].name, "main");
}
