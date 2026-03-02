#[derive(Clone)]
struct Team {
    id: i32,
    name: String,
    score: i32,
    is_allowed_to_buzz: bool
}

enum BuzzerState {
    Idle,
    LockedByTeam(i32),
    LockedForAll
}

struct Buzzer {
    state: BuzzerState
}

impl Team {
    fn new_team(id: i32, name: String) -> Self {
        Team {
            id,
            name,
            score: 0,
            is_allowed_to_buzz: true
        }
    }

    fn update_score(&mut self, points: i32) {
        self.score += points;
    }

    fn set_buzzing_permission(&mut self, permission: bool) {
        self.is_allowed_to_buzz = permission;
    }
}

impl Buzzer {
    fn buzz(&mut self, team: Team) {
        match self.state {
            BuzzerState::Idle => self.state = BuzzerState::LockedByTeam(team.id),
            BuzzerState::LockedByTeam(t) => println!("Buzzer is already locked by team n°{}!", t),
            BuzzerState::LockedForAll => println!("Buzzer is locked for all teams!")
        }
    }

    fn reset(&mut self) {
        self.state = BuzzerState::Idle;
    }

    fn lock_for_all(&mut self) {
        self.state = BuzzerState::LockedForAll;
    }

    fn unlock_for_all(&mut self) {
        self.state = BuzzerState::Idle;
    }
}

struct Game {
    buzzer: Buzzer,
    teams: Vec<Team>
}

impl Game {
    fn new_game() -> Self {
        Game {
            buzzer: Buzzer { state: BuzzerState::Idle },
            teams: Vec::new()
        }
    }

    fn add_team(&mut self, team: Team) {
        self.teams.push(team);
    }

    fn find_team_by_id(&self, id: i32) -> Option<&Team> {
        self.teams.iter().find(|&team| team.id == id)
    }

    fn handle_buzz(&mut self, team: Team) {
        if team.is_allowed_to_buzz {
            self.buzzer.buzz(team);
        } else {
            println!("Team {} is not allowed to buzz!", team.name);
        }
    }
}

pub fn test_buzzer() {
    let mut game = Game::new_game();
    let team1 = Team::new_team(1, "Team A".to_string());
    let team2 = Team::new_team(2, "Team B".to_string());

    game.add_team(team1);
    game.add_team(team2);

    if let Some(team) = game.find_team_by_id(1) {
        game.handle_buzz(team.clone());
    }

    if let Some(team) = game.find_team_by_id(2) {
        game.handle_buzz(team.clone());
    }

    game.buzzer.reset();

    if let Some(team) = game.find_team_by_id(2) {
        game.handle_buzz(team.clone());
    }
}

