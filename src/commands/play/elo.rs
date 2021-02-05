use super::*;

pub type Elo = f64;

fn get(server: u64, player: u64, game: &str) -> Result<Elo> {
    let player = format!("{}", player);
    let server = format!("{}", server);
    let db = db()?;
    let elo: Elo = db
        .query_row(
            &format!(
                "SELECT elo FROM {} WHERE player = ?2 AND server = ?1",
                elo_table(game)
            ),
            params!(server, player),
            |row| row.get(0),
        )
        .unwrap_or(1200.0);
    Ok(elo)
}

fn set(server: u64, player: u64, game: &str, elo: Elo) -> Result<()> {
    let player = format!("{}", player);
    let server = format!("{}", server);

    let db = db()?;
    let affected = db.execute(
        &format!(
            "UPDATE {} SET elo = ?3 WHERE player=?1 AND server=?2;",
            elo_table(game)
        ),
        params!(player, server, elo),
    )?;

    if affected == 0 {
        db.execute(
            &format!(
                "INSERT INTO {} (server, player, elo) VALUES (?1, ?2, ?3);",
                elo_table(game)
            ),
            params!(server, player, elo),
        )?;
    }
    Ok(())
}

pub fn process_game(
    game_name: &str,
    server: u64,
    player_id: &[u64],
    winner: Option<usize>,
) -> Result<()> {
    let mut elo = Vec::new();
    for &p in player_id.iter() {
        elo.push(get(server, p, game_name)?);
    }

    // expected score for player 0
    let exp0 = 1.0 / (1.0 + 10.0_f64.powf((elo[1] - elo[0]) / 400.0));

    // actual score for player 0
    let score0 = winner.map_or(0.5, |p| 1.0 - p as f64);

    const K: f64 = 40.0;

    // calculate elo addition/subtraction
    let d_elo = K * (score0 - exp0);

    // update elo
    set(server, player_id[0], game_name, elo[0] + d_elo)?;
    set(server, player_id[1], game_name, elo[1] - d_elo)?;

    Ok(())
}
