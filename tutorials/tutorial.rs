use bevy::prelude::*;
//brings essentials from bevy


//App contains World
//outer loop
//only for structure

/*Resource
Timer for Greets every 2 secs
*/
#[derive(Resource)]
struct GreetTimer(Timer);

/*
Component
Person
*/

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Name(String);

//----Systems

/* Starup System

Adds peaople to the world 
 */

fn add_people(mut commands: Commands) {
    commands.spawn((Person, Name("Elaina Proctor".to_string())));
    commands.spawn((Person, Name("Renzo Hume".to_string())));
    commands.spawn((Person, Name("Zayna Nieves".to_string())));
}


/*System: Query
greets people every x quantity of time
Input: current time, time since last update
all entities with the name and person component
Borrows the input
*/
fn greet_people(time: Res<Time>, mut timer: ResMut<GreetTimer>, query: Query<&Name, With<Person>>) {
    // update our timer with the time elapsed since the last update
    // if that caused the timer to finish, we say hello to everyone
    if timer.0.tick(time.delta()).just_finished() {
        for name in &query {
            println!("hello {}!", name.0);
        }
    }
}


/*System: Query Mutable
changes name of all entities with name and person component
 */
fn update_people(mut query: Query<&mut Name, With<Person>>) {
    for mut name in &mut query {
        if name.0 == "Elaina Proctor" {
            name.0 = "Elaina Hume".to_string();
            break; // We don't need to change any other names.
        }
    }
}


/*Plugin: Hello
 */

pub struct HelloPlugin;

impl Plugin for HelloPlugin {
    fn build(&self, app: &mut App) {
        // add things to your app here
        app.insert_resource(GreetTimer(Timer::from_seconds(2.0, TimerMode::Repeating)));
        app.add_systems(Startup, add_people);
        app.add_systems(Update, (update_people, greet_people).chain());
    }
}

/*
App provides us with tools for:

    Initializing resources in the World to store globally available data that we only need a single copy of.
    Adding systems to our Schedule, which can read and modify resources and our entities' components, according to our game logic.
    Importing other blocks of App-modifying code using Plugins.

    Note: Systems run in parallel if posible
    if u want them to run in a specific order then chain
    if the order doesnt mater leave without chain for paralelization.

*/
fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(HelloPlugin) //event loop and window management
    .run();
}