#![windows_subsystem = "windows"]

use bevy::{
    prelude::*,
    sprite::MaterialMesh2dBundle
};
use bevy::sprite::Anchor;
use bevy::window::WindowResized;
use rand::Rng;

enum State {
    Clicked,
    Flagged,
    Untouched,
    JustClicked
}
enum Status {
    WON,
    LOST,
    RUNNING,
    START,
    RESET
}
#[derive(Component)]
struct Square {
    mine:bool,
    point:Point,
    state:State,
    number:i32,
    flags:i32
}
#[derive(Component)]
struct GameController {
    status: Status,
    rows:i32,
    columns:i32,
    number_mines:i32,
    layers:i32
}
#[derive(Component)]
struct SquareText {
    point:Point
}
#[derive(Component)]
struct MineCount {
    mines_left:i32
}
#[derive(Component)]
struct Timer {
    time:f32
}
struct Point {
    x:i32,
    y:i32
}

fn main(){
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (spawn_camera, spawn_mine_count, spawn_timer, spawn_game_controller))
        .add_systems(Update, (on_window_resize, square_update, restart, square_text_update, mine_count_update, update_timer, game_controller_update))
        .run();
}
fn spawn_camera(mut commands: Commands){
    commands.spawn(Camera2dBundle::default());
}
fn spawn_timer(q_window: Query<&Window>, mut commands: Commands){
    commands.spawn((
        Timer{time:0.0},
        Text2dBundle {
            text: Text::from_section(
                "0.00",
                TextStyle {
                    font_size:30.0,
                    color:Color::WHITE,
                    ..default()}
            ),
            text_anchor: Anchor::TopRight,
            transform: Transform::from_translation(Vec3::new(q_window.single().width()/2.0, q_window.single().height()/2.0, 1.0)),
            ..default()
        }
    ));
}
fn spawn_game_controller (mut commands: Commands){
    commands.spawn(
        GameController {status: Status::RESET, columns:30, rows:16, number_mines:99, layers:1}
    );
}
fn spawn_mine_count (q_window: Query<&Window>, mut commands: Commands){
    commands.spawn((
        MineCount{mines_left:99},
        Text2dBundle {
            text: Text::from_section(
                    "99: Expert",
                    TextStyle {
                        font_size:30.0,
                        color:Color::WHITE,
                        ..default()}
                    ),
            text_anchor: Anchor::TopLeft,
            transform: Transform::from_translation(Vec3::new(-q_window.single().width()/2.0, q_window.single().height()/2.0, 1.0)),
            ..default()
        }
    ));
}
fn mine_count_update(mut mine_query: Query<(&mut MineCount, &mut Text)>, square_query: Query<&Square>, gc_query: Query<&GameController>){
    let mut i = 0;
    for square in square_query.iter(){
        if let State::Flagged = square.state {
            i += 1;
        }
    }
    let mines = gc_query.single().number_mines;
    let left = mines-i;
    let dif = if mines==99 {"Expert: "} else if mines==40 {"Intermediate: "} else {"Beginner: "};
    mine_query.single_mut().0.mines_left = mines-i;
    let s = format!("{}{}", dif, left);
    mine_query.single_mut().1.sections[0].value = String::from(s);

}
fn update_timer (time: Res<Time>, mut timer_query: Query<(&mut Timer, &mut Text)>, gc_query: Query<&GameController>){
    let mut timing = true;
    for (mut timer, mut text) in timer_query.iter_mut() {
        match gc_query.single().status {
            Status::LOST => {
                timing = false;
                text.sections[0].style.color = Color::RED;
            },
            Status::WON => {
                timing = false;
                text.sections[0].style.color = Color::GREEN;
            },
            Status::RUNNING => {
                text.sections[0].style.color = Color::WHITE;
            },
            _ => {
                timing = false;
                text.sections[0].style.color = Color::WHITE;
                text.sections[0].value = String::from("0.00");
                timer.time = 0.0;
            }
        }
        if timing {
            timer.time += time.delta_seconds();
            text.sections[0].value = ((timer.time*100.0).round()/100.0).to_string();
        }
    }
}
fn on_window_resize(
    mut resize_events: EventReader<WindowResized>, 
    mut timer_query: Query<(&Timer, &mut Transform), (Without<Square>, Without<SquareText>, Without<MineCount>)>,
    mut query: Query<(&Square, &mut Transform), Without<Text>>, 
    mut mine_count_query: Query<&mut Transform, (With<Text>, With<MineCount>, Without<Square>, Without<SquareText>)>, 
    mut square_text: Query<(&SquareText, &mut Transform), (With<Text>, Without<Square>)>,
    gc_query: Query<&GameController>){
    for e in resize_events.read() {
        let size = if (e.height-35.0)/(gc_query.single().rows as f32) < (e.width)/(gc_query.single().columns as f32) {(e.height-35.0)/gc_query.single().rows as f32} else {e.width/gc_query.single().columns as f32};
        for (square, mut transform) in query.iter_mut() {
            transform.translation.x = 1.0 + size * ((1.0 - gc_query.single().columns as f32) / 2.0 + square.point.x as f32);
            transform.translation.y = 1.0 + -35.0/2.0 -size*gc_query.single().rows as f32/2.0 + size/2.0 + size * square.point.y as f32;
            transform.scale.x = size - 2.0;
            transform.scale.y = size - 2.0;
        }
        for mut transform in mine_count_query.iter_mut(){
            transform.translation.x = -e.width/2.0;
            transform.translation.y = e.height/2.0;
        }
        for (square_text, mut transform) in square_text.iter_mut(){
            transform.translation.x = size * ((1.0 - gc_query.single().columns as f32) / 2.0 + square_text.point.x as f32);
            transform.translation.y = -35.0/2.0 -size*gc_query.single().rows as f32/2.0 +size/2.0 + size * square_text.point.y as f32;
            transform.scale.x = (size - 2.0)/100.0;
            transform.scale.y = (size - 2.0)/100.0;
        }
        for (_, mut transform) in timer_query.iter_mut(){
            transform.translation.x = e.width / 2.0;
            transform.translation.y = e.height / 2.0;
        }
    }
}
fn game_controller_update (
    mut gc_query: Query<&mut GameController>,
    sq_query: Query<&Square>,
    keys: Res<Input<KeyCode>>) {

    let mut num_clicked = 0;
    for mut controller in gc_query.iter_mut(){
        if keys.just_pressed(KeyCode::Space) {
            controller.status = Status::RESET;
            break;
        } else if keys.just_pressed(KeyCode::E) {
            controller.rows = 16;
            controller.columns = 30;
            controller.number_mines = 99;
            controller.status = Status::RESET;
            break;
        } else if keys.just_pressed(KeyCode::I) {
            controller.rows = 16;
            controller.columns = 16;
            controller.number_mines = 40;
            controller.status = Status::RESET;
            break;
        } else if keys.just_pressed(KeyCode::B) {
            controller.rows = 9;
            controller.columns = 9;
            controller.number_mines = 10;
            controller.status = Status::RESET;
            break;
        }
        for square in sq_query.iter() {
            match square.state {
                State::Clicked => {
                    if square.mine { controller.status = Status::LOST }
                    num_clicked +=1;
                },
                State::JustClicked => {
                    if square.mine { controller.status = Status::LOST }
                    num_clicked +=1;
                }
                _ => {}
            }
        }
        match controller.status {
            Status::RUNNING => {
                if num_clicked == controller.columns*controller.rows-controller.number_mines {
                    controller.status = Status::WON;
                }
            },
            Status::START => {
                if num_clicked >0 {
                    controller.status = Status::RUNNING;
                }
            },
            Status::RESET => {},
            _=>{if num_clicked==0 {controller.status = Status::START}}
        }
    }
}
fn square_text_update (query: Query<&Square>, mut square_text_query: Query<(&SquareText, &mut Visibility, &mut Text)>) {
    for (square_text, mut visibility, mut text) in square_text_query.iter_mut(){
        for square in query.iter() {
            let x = (square.point.x - square_text.point.x).abs();
            let y = (square.point.y - square_text.point.y).abs();
            if x == 0 && y == 0 {
                match square.state {
                    State::JustClicked => {
                        if square.number >0 {
                            *visibility = Visibility::Visible;
                        }
                    },
                    State::Clicked => {
                        if square.number >0 {
                            *visibility = Visibility::Visible;
                        }
                    },
                    _ => {
                        *visibility = Visibility::Hidden;
                    }
                }
                text.sections[0].value = square.number.to_string();
            }
        }
    }
}
fn square_update(
    q_windows: Query<&Window>,
    mut query: Query<(&mut Square, &Transform, &Handle<ColorMaterial>)>,
    gc_query: Query<&GameController>,
    buttons: Res<Input<MouseButton>>,
    mut materials: ResMut<Assets<ColorMaterial>>) {
    let mouse_x = if let Some(position) = q_windows.single().cursor_position() { position.x - q_windows.single().width() / 2.0 } else { q_windows.single().width() };
    let mouse_y = if let Some(position) = q_windows.single().cursor_position() { q_windows.single().height() / 2.0 - position.y } else { q_windows.single().height() };
    let mut gen_mines = false;
    let mut start = Point { x: 31, y: 17 };
    let mut coord: Vec<Point> = vec![];
    let mut reset = false;
    for (mut square, transform, material) in query.iter_mut() {
        let lower = transform.translation.y-transform.scale.y/2.0;
        let upper = transform.translation.y+transform.scale.y/2.0;
        let left = transform.translation.x-transform.scale.x/2.0;
        let right = transform.translation.x+transform.scale.x/2.0;
        let mut mouse_in = false;
        square.flags = 0;
        if mouse_x >= left && mouse_x < right && mouse_y >= lower && mouse_y <upper {
            mouse_in = true;
        }
        let mat = materials.get_mut(material).unwrap();
        match square.state {
            State::Untouched => {
                mat.color = Color::rgb(0.45, 0.1, 0.45);
            },
            State::Flagged => {
                mat.color = Color::rgb(1.0, 0.0, 0.75);
            },
            _ => {
                mat.color = if square.mine {Color::rgb(1.0, 0.4, 0.0)} else {Color::rgb(0.0, 0.1, 0.3)};
            }
        }
        match gc_query.single().status {
            Status::RUNNING=> {
                match square.state {
                    State::JustClicked => {
                        if square.number == 0 {
                            coord.push(Point {x:square.point.x, y:square.point.y});
                        }
                        square.state = State::Clicked;
                    }
                    State::Clicked => {
                        if mouse_in && buttons.just_pressed(MouseButton::Left){
                            coord.push(Point {x:square.point.x, y:square.point.y});
                        }
                    },
                    State::Untouched => {
                        if buttons.just_pressed(MouseButton::Left) && mouse_in {
                            square.state = State::JustClicked;
                        }
                        if buttons.just_pressed(MouseButton::Right) && mouse_in {
                            square.state = State::Flagged;
                        }
                    },
                    State::Flagged => {
                        if buttons.just_pressed(MouseButton::Right) && mouse_in {
                            square.state = State::Untouched;
                        }
                    }
                }
            },
            Status::START => {
                if buttons.just_pressed(MouseButton::Left) && mouse_in {
                    square.state = State::JustClicked;
                    gen_mines = true;
                    start.x  = square.point.x;
                    start.y = square.point.y;
                }
            },
            Status::LOST=> {
                if buttons.just_pressed(MouseButton::Left){
                    reset = true;
                }
                match square.state {
                    State::Flagged => {
                        if !square.mine {mat.color = Color::BLACK}
                    },
                    State::Untouched =>{
                        square.state = State::JustClicked;
                    },
                    _=>{}
                }
            },
            Status::WON=> {
                if buttons.just_pressed(MouseButton::Left){
                    reset = true;
                }
            },
            Status::RESET=> {
                reset = true;
            }
        }
    }
    if reset {
        for (mut square, _, _) in query.iter_mut() {
            square.state = State::Untouched;
            square.mine = false;
            square.number = 0;
            square.flags = 0;
        }
    }
    if gen_mines{
        let mut rng = rand::thread_rng();
        let mut i = 0;
        loop {
            let x = rng.gen_range(0..gc_query.single().columns);
            let y = rng.gen_range(0..gc_query.single().rows);
            for (mut square, _, _) in query.iter_mut(){
                if x==square.point.x && y==square.point.y && !square.mine && ((x-start.x).abs()>gc_query.single().layers || (y-start.y).abs()>gc_query.single().layers) {
                    square.mine = true;
                    square.number = -1;
                    i += 1;
                    if i==gc_query.single().number_mines {break}
                }
            }
            if i==gc_query.single().number_mines {break}
        }
        let mut iter = query.iter_combinations_mut();

        while let Some([
                       (mut square, _,_),
                       (mut square2, _,_)
                       ]) = iter.fetch_next() {
            let x = (square.point.x-square2.point.x).abs();
            let y = (square.point.y-square2.point.y).abs();
            if x+y ==0 || x>gc_query.single().layers || y>gc_query.single().layers {continue}
            if square2.mine && !square.mine {square.number += 1}
            if square.mine && !square2.mine {square2.number += 1}
        }
    }
    let mut iter = query.iter_combinations_mut();
    while let Some([
                   (mut square, _,_),
                   (mut square2, _,_)
                   ]) = iter.fetch_next() {
        let x = (square.point.x-square2.point.x).abs();
        let y = (square.point.y-square2.point.y).abs();
        if x+y==0 || x>gc_query.single().layers || y>gc_query.single().layers {continue}
        match square2.state {
            State::Flagged => {
                square.flags += 1;
            },
            _=>{}
        }
        match square.state {
            State::Flagged => {
                square2.flags += 1;
            },
            _=>{}
        }
    }
    for point in coord {
        let mut iter2 = query.iter_combinations_mut();
        while let Some([
                       (mut square, _,_),
                       (mut square2, _,_)
                       ]) = iter2.fetch_next() {
            let x = (square.point.x-square2.point.x).abs();
            let y = (square.point.y-square2.point.y).abs();
            if x+y ==0 || x>gc_query.single().layers || y>gc_query.single().layers {continue}
            if point.x == square.point.x && point.y == square.point.y && (square.flags==square.number || square.number==0){
                match square2.state {
                    State::Untouched => {
                        square2.state = State::JustClicked;
                    },
                    _=>{}
                }
            } else if point.x == square2.point.x && point.y == square2.point.y && (square2.flags==square2.number || square2.number==0){
                match square.state {
                    State::Untouched => {
                        square.state = State::JustClicked;
                    },
                    _=>{}
                }
            }
        }
    }
}
fn restart(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>, mut gc_query: Query<&mut GameController>, win_query: Query<&Window>, square_query: Query<Entity, With<Square>>, st_query: Query<Entity, With<SquareText>>){
    match gc_query.single().status {
        Status::RESET => {
            for entity in square_query.iter() {
                commands.entity(entity).despawn_recursive();
            }
            for entity in st_query.iter() {
                commands.entity(entity).despawn_recursive();
            }
            let height = win_query.single().height();
            let width = win_query.single().width();
            let size = if (height-35.0)/(gc_query.single().rows as f32) < (width)/(gc_query.single().columns as f32) {(height-35.0)/gc_query.single().rows as f32} else {width/ gc_query.single().columns as f32 };
            for i in 0..gc_query.single().columns {
                for j in 0..gc_query.single().rows {
                    commands.spawn((
                        SquareText {point:Point {x:i, y:j}},
                        Text2dBundle {
                            text: Text::from_section(
                                "",
                                TextStyle {
                                    font_size:100.0,
                                    color:Color::WHITE,
                                    ..default()}
                            ),
                            transform: Transform::from_translation(Vec3::new(1.0+size*((1.0-gc_query.single().columns as f32)/2.0+i as f32), -35.0/2.0 -size*gc_query.single().rows as f32/2.0 +size/2.0 + size * j as f32, 1.0))
                                .with_scale(Vec3::new((size-2.0)/100.0, (size-2.0)/100.0, 1.0)),
                            visibility: Visibility::Hidden,
                            text_anchor: Anchor::Center,
                            ..default()
                        }
                    ));
                    commands.spawn((
                        Square {
                            mine:false,
                            state:State::Untouched,
                            point:Point {x:i, y:j},
                            number:0,
                            flags:0
                        },
                        MaterialMesh2dBundle {
                            mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
                            material: materials.add(ColorMaterial::from(Color::GRAY)),
                            transform: Transform::from_translation(Vec3::new(1.0+size*((1.0-gc_query.single().columns as f32)/2.0+i as f32), 1.0-35.0/2.0 -size*gc_query.single().rows as f32/2.0 +size/2.0 + size * j as f32, 0.0))
                                .with_scale(Vec3::new(size-2.0,size-2.0,1.0)),
                            ..default()
                        }
                    ));
                }
            }
            gc_query.single_mut().status = Status::START;
        },
        _=>{}
    }
}