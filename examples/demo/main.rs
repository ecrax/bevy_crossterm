use bevy::prelude::*;
use bevy_crossterm::prelude::*;

use bevy_asset::LoadedFolder;
use bevy_crossterm::CrosstermKeyEventWrapper;
use std::default::Default;

use bevy::log::LogPlugin;
mod animation;
mod colors;
mod finale;
mod sprites;
mod title;

#[derive(Clone, States, Default, Eq, PartialEq, Hash, Debug)]
pub enum GameState {
    #[default]
    Loading,
    Title,
    Sprites,
    Colors,
    Animation,
    Finale,
}

impl GameState {
    pub fn next_state(&self) -> Option<GameState> {
        use GameState::*;
        match self {
            Loading => Some(Title),
            Title => Some(Sprites),
            Sprites => Some(Colors),
            Colors => Some(Animation),
            Animation => Some(Finale),
            Finale => None,
        }
    }
}

pub fn main() {
    // Window settings must happen before the crossterm Plugin
    let mut settings = CrosstermWindowSettings::default();
    settings.set_title("bevy_crossterm demo");

    App::new()
        .insert_resource(settings)
        .add_plugins(bevy_app::ScheduleRunnerPlugin::run_loop(
            std::time::Duration::from_millis(16),
        ))
        .add_plugins(
            DefaultPlugins
                .set(LogPlugin {
                    filter: "off".into(),
                    level: bevy::log::Level::ERROR,
                    ..default()
                })
                .set(TaskPoolPlugin {
                    task_pool_options: TaskPoolOptions::with_num_threads(1),
                }),
        )
        .add_plugins(CrosstermPlugin)
        .init_state::<GameState>()
        .add_systems(Startup, loading_system)
        .add_systems(
            Update,
            check_for_loaded.run_if(in_state(GameState::Loading)),
        )
        .add_systems(OnEnter(GameState::Title), title::setup)
        .add_systems(
            Update,
            just_wait_and_advance.run_if(in_state(GameState::Title)),
        )
        .add_systems(OnExit(GameState::Title), simple_teardown)
        .add_systems(OnEnter(GameState::Sprites), sprites::setup)
        .add_systems(
            Update,
            just_wait_and_advance.run_if(in_state(GameState::Sprites)),
        )
        .add_systems(OnExit(GameState::Sprites), simple_teardown)
        .add_systems(OnEnter(GameState::Colors), colors::setup)
        .add_systems(
            Update,
            just_wait_and_advance.run_if(in_state(GameState::Colors)),
        )
        .add_systems(OnExit(GameState::Colors), simple_teardown)
        .add_systems(OnEnter(GameState::Animation), animation::setup)
        .add_systems(
            Update,
            animation::update.run_if(in_state(GameState::Animation)),
        )
        .add_systems(OnExit(GameState::Animation), simple_teardown)
        .add_systems(OnEnter(GameState::Finale), finale::setup)
        .add_systems(
            Update,
            just_wait_and_advance.run_if(in_state(GameState::Finale)),
        )
        .add_systems(OnExit(GameState::Finale), simple_teardown)
        .run();
}

#[derive(Resource)]
struct CrosstermAssets(Handle<LoadedFolder>);

fn loading_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut cursor: ResMut<Cursor>,
) {
    cursor.hidden = true;

    // Load the assets we want
    let handle = asset_server.load_folder("demo");

    commands.insert_resource(CrosstermAssets(handle));
}

// This function exists solely because bevy's asset loading is async.
// We need to wait until all assets are loaded before we do anything with them.
fn check_for_loaded(
    asset_server: Res<AssetServer>,
    handles: Res<CrosstermAssets>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if asset_server.is_loaded_with_dependencies(&handles.0) {
        let state = state.next_state().unwrap();
        next_state.set(state);
    }
}

// Helper function to see if there was a key press this frame
pub fn detect_keypress(mut keys: EventReader<CrosstermKeyEventWrapper>) -> bool {
    let res = !keys.is_empty();
    keys.clear();
    res
}

// Simple update function that most screens will use
pub fn just_wait_and_advance(
    mut app_exit: ResMut<Events<bevy::app::AppExit>>,
    keys: EventReader<CrosstermKeyEventWrapper>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if detect_keypress(keys) {
        if let Some(state) = state.next_state() {
            next_state.set(state);
        } else {
            app_exit.send(bevy::app::AppExit);
        }
    }
}

// Looks for an entity resource and then de-spawns that entity and all it's children
pub fn simple_teardown(mut commands: Commands, query: Query<Entity, With<Position>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
