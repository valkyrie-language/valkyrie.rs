use clr_backend::msil::MsilParser;

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

    assert_eq!(methods.len(), 2, "应解析出 2 个方法");
    assert_eq!(methods[0].name, "Add");
    assert_eq!(methods[1].name, "Main");
}

#[test]
fn empty_input_produces_empty_list() {
    let methods = MsilParser::parse_methods("");
    assert!(methods.is_empty());
}

#[test]
fn unbalanced_block_is_captured_to_eof() {
    let methods = MsilParser::parse_methods(
        r#".method public void Foo() cil managed
{
  ret
"#,
    );
    assert_eq!(methods.len(), 1, "不闭合的方法仍应被收录");
    assert!(!methods[0].body.is_empty());
}
