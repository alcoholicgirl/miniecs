use miniecs::*;

#[derive(Component)]
struct People {
    name: &'static str,
    age: u8,
}

fn main() {
    let mut scheduler = Scheduler::new();
    let mut world = World::new();
    let tom = world.spawn();
    world.add_component(
        tom,
        People {
            name: "Tom",
            age: 20,
        },
    );

    scheduler.push(|people: &People| {
        println!("Hi! My name is {} and I'm {} y/o.", people.name, people.age);
    });
    scheduler.schedule(world.get_handle());
}
