mod parser {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/msil/parser.rs"));

    const SAMPLE_MSIL: &str = r#".assembly extern mscorlib {}
.assembly hello {}

.class public auto ansi beforefieldinit Hello
       extends [mscorlib]System.Object
{
  .method public hidebysig static int32 Add(int32 a, int32 b) cil managed
  {
    .maxstack  2
    ldarg.0
    ldarg.1
    add
    ret
  }

  .method public hidebysig static void Main() cil managed
  {
    .entrypoint
    .maxstack  1
    ldc.i4.1
    ret
  }
}
"#;

    #[test]
    fn parses_method_signatures() {
        let methods = MsilParser::parse_methods(SAMPLE_MSIL);
        assert_eq!(methods.len(), 2);
        assert_eq!(methods[0].name, "Add");
        assert_eq!(methods[1].name, "Main");
    }

    #[test]
    fn extracts_method_names_from_signatures() {
        assert_eq!(extract_method_name("public static int32 Add(int32, int32)"), "Add");
        assert_eq!(extract_method_name("public static void Main()"), "Main");
        assert_eq!(extract_method_name("public hidebysig instance void .ctor()"), ".ctor");
    }

    #[test]
    fn extracts_signatures_from_method_lines() {
        let sig = extract_signature(".method public hidebysig static int32 Add(int32 a, int32 b) cil managed {");
        assert!(!sig.contains('{'));
        assert!(sig.contains("Add"));
    }

    #[test]
    fn finds_block_end_correctly() {
        let lines: Vec<&str> = SAMPLE_MSIL.lines().collect();
        let add_start = lines.iter().position(|line| line.trim_start().starts_with(".method") && line.contains("Add(")).unwrap();
        let (end, balanced) = find_block_end(&lines, add_start);
        assert!(balanced);
        assert!(end > add_start);
    }

    #[test]
    fn handles_nested_braces() {
        let source = r#".method public void Foo() cil managed
{
  .emitbyte 0x00
  {
    nested
  }
  ret
}"#;
        let lines: Vec<&str> = source.lines().collect();
        let start = lines.iter().position(|line| line.trim_start().starts_with(".method")).unwrap();
        let (end, balanced) = find_block_end(&lines, start);
        assert!(balanced);
        assert_eq!(lines[end].trim(), "}");
    }

    #[test]
    fn empty_input_produces_empty_list() {
        let methods = MsilParser::parse_methods("");
        assert!(methods.is_empty());
    }

    #[test]
    fn unbalanced_block_is_captured_to_eof() {
        let methods = MsilParser::parse_methods(".method public void Foo() cil managed\n{\n  ret\n");
        assert_eq!(methods.len(), 1);
        assert!(!methods[0].body.is_empty());
    }
}
