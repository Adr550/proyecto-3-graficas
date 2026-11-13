mod arcade;
mod characters;
mod client;
mod connect_screen;
mod hub;
mod quoridor;
mod rhythm;
mod studio;
mod view;
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if let Some(i) = args.iter().position(|a| a == "--render-characters") {
        return characters::export_art(std::path::Path::new(
            args.get(i + 1)
                .map(String::as_str)
                .unwrap_or("renders/characters"),
        ));
    }
    if args.iter().any(|a| a == "--simulate") {
        return quoridor::command_line();
    }
    if let Some(i) = args.iter().position(|a| a == "--render") {
        let dir = std::path::Path::new(args.get(i + 1).map(String::as_str).unwrap_or("renders"));
        hub::export(dir)?;
        characters::export_portraits(&dir.join("orbit-club-portraits.png"))?;
        quoridor::command_line()?;
        return Ok(());
    }
    let (mut rl, t) = raylib::init()
        .size(1280, 850)
        .title("Orbit Club / Meet your planet rivals")
        .resizable()
        .msaa_4x()
        .build();
    rl.set_window_min_size(1080, 720);
    rl.set_exit_key(None);
    rl.set_target_fps(60);
    let cast = characters::Cast::load(&mut rl, &t)?;
    if let Some(i) = args.iter().position(|a| a == "--capture-suite") {
        let dir = std::path::Path::new(args.get(i + 1).map(String::as_str).unwrap_or("captures"));
        return hub::captures(&mut rl, &t, dir, &cast);
    }
    if args.iter().any(|a| a == "--beat") {
        rhythm::play(&mut rl, &t, &cast)?;
    } else if args.iter().any(|a| a == "--studio" || a == "--gallery") {
        studio::run(&mut rl, &t, &cast)?;
    } else if args.iter().any(|a| a == "--quoridor") {
        quoridor::play(&mut rl, &t, &cast)?;
    } else if args.iter().any(|a| a == "--chess") {
        arcade::chess(&mut rl, &t, &cast, characters::Character::Star)?;
    } else if args.iter().any(|a| a == "--blast") {
        arcade::bomber(&mut rl, &t, &cast)?;
    } else if args.iter().any(|a| a == "--connect4") {
        connect_screen::play(&mut rl, &t, &cast)?;
    } else {
        hub::run(&mut rl, &t, &cast)?;
    }
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("Orbit Club: {e}");
        std::process::exit(1);
    }
}
