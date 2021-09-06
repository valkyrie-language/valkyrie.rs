use legion_pool::ValkyrieTypeGraph;

#[test]
fn main() {
    let mut graph = ValkyrieTypeGraph::new(999);

    let animal = graph.register_class("std::number::i32");
    let animal = graph.register_class("std::number::i64");
    let animal = graph.register_class("std::number::f32");
    let animal = graph.register_class("std::number::f64");


    let animal = graph.register_class("Animal");
    animal.add_instance_of("Swim");
    animal.add_instance_of("Yell");

    let anonymous = graph.register_class("Anonymous<x:i32,y:i32>");
    animal.add_instance_of("Anonymous<x:i32,y:i32>");


    let swim = graph.register_class("Swim");
    let yell = graph.register_class("Yell");

    let cat = graph.register_class("Cat");
    cat.add_parent("Yell");
    cat.add_parent("Animal");

    let dog = graph.register_class("Dog");
    dog.add_parent("Animal");
    dog.add_parent("Swim");

    let fish = graph.register_class("Fish");
    fish.add_parent("Animal");
    fish.add_parent("Swim");

    match graph.linearize() {
        Ok(linearized_graph) => {
            println!("{:#?}", linearized_graph);
        }
        Err(e) => println!("Error: {:?}", e),
    }
}
