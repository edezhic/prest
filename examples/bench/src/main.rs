use prest::*;
use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Instant,
};

const START_TABLE_SIZE: u64 = 100_000;

const SPAWN_INTERVAL: u32 = 100;

const READS_PER_SPAWN: u64 = 100;
const UPDATES_PER_SPAWN: u64 = 10;
const SAVES_PER_SPAWN: u64 = 1;

#[derive(Clone, Storage, Serialize, Deserialize)]
struct Entry {
    pub id: u64,
    pub uuid: Uuid,
    #[unique]
    pub unique: String,
    pub optional: Option<String>,
    pub list: Vec<bool>,
    pub state: State,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
enum State {
    Simple,
    Complex { a: String, b: Option<f64> },
    Counter(u64),
}

impl Entry {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            uuid: Uuid::now_v7(),
            unique: id.to_string(),
            optional: Some("can be None instead".to_owned()),
            state: State::Complex {
                a: "Lorem ipsum dolor sit amet, consectetur adipiscing elit".to_owned(),
                b: Some(123.456),
            },
            list: vec![true, false],
        }
    }
}

static ID_OFFSET: AtomicU64 = AtomicU64::new(1);
static ROWS_COUNT: AtomicU64 = AtomicU64::new(0);

#[init]
async fn main() -> Result {
    prepare().await?;

    info!(
        "spawning {:.0} reads per sec, {:.0} updates per sec and adding {:.0} new entries per sec",
        1000.0 / SPAWN_INTERVAL as f32 * READS_PER_SPAWN as f32,
        1000.0 / SPAWN_INTERVAL as f32 * UPDATES_PER_SPAWN as f32,
        1000.0 / SPAWN_INTERVAL as f32 * SAVES_PER_SPAWN as f32,
    );

    RT.every(SPAWN_INTERVAL).milliseconds().spawn(|| async {
        for _ in 0..READS_PER_SPAWN {
            RT.once(get_random());
        }
        for _ in 0..UPDATES_PER_SPAWN {
            RT.once(update());
        }
        for _ in 0..SAVES_PER_SPAWN {
            RT.once(save());
        }
    });

    route("/", get("check out monitoring in the admin panel (/admin)"))
        .run()
        .await?;

    OK
}

async fn prepare() -> Result {
    // clear the DB
    let start = Instant::now();
    DB.nuke().await?;
    info!("nuked database in {:.0}ms", ms(&start));

    // save rows
    let mut set = JoinSet::new();
    for _ in 1..=START_TABLE_SIZE {
        set.spawn(save());
    }
    let start = Instant::now();
    set.join_all().await;
    info!(
        "seeded table with {START_TABLE_SIZE} rows in {:.0}ms",
        ms(&start)
    );

    // KV read them
    let mut set = JoinSet::new();
    for i in 1..=START_TABLE_SIZE {
        set.spawn(get_by_pkey(i));
    }
    let start = Instant::now();
    set.join_all().await;
    info!(
        "finished {START_TABLE_SIZE} KV reads in {:.0}ms",
        ms(&start)
    );

    // SQL read them
    let mut set = JoinSet::new();
    for i in 1..=START_TABLE_SIZE {
        set.spawn(sql_get_by_pkey(i));
    }
    let start = Instant::now();
    set.join_all().await;
    info!(
        "finished {START_TABLE_SIZE} SQL reads in {:.0}ms",
        ms(&start)
    );
    OK
}

async fn get_random() {
    let id = random_id();
    match Entry::get_by_pkey(id).await {
        Ok(Some(e)) => assert!(e.id == id),
        Ok(None) => error!("failed to read one: entry not found (id = {id})"),
        Err(e) => error!("failed to get_by_peyk: {e}"),
    };
}

async fn get_by_pkey(pk: u64) {
    match Entry::get_by_pkey(pk).await {
        Ok(Some(e)) => assert!(e.id == pk),
        Ok(None) => error!("failed to read one: entry not found (id = {pk})"),
        Err(e) => error!("failed to get_by_pkey: {e}"),
    };
}

async fn sql_get_by_pkey(pk: u64) {
    match DB
        .read_sql_rows::<Entry>(&format!("select * from 'Entry' where id = {pk};"))
        .await
    {
        Ok(entries) => assert!(entries[0].id == pk),
        Err(e) => error!("failed to read_sql_rows: {e}"),
    }
}

async fn save() {
    let id = ID_OFFSET.fetch_add(1, Ordering::SeqCst);
    match Entry::new(id).save().await {
        Ok(_) => {
            ROWS_COUNT.fetch_add(1, Ordering::SeqCst);
        }
        Err(e) => error!("failed to save: {e}"),
    }
}

async fn update() {
    let id = random_id();

    let Some(mut entry) = Entry::get_by_pkey(id)
        .await
        .expect("get_by_pkey with existing PK value should be fine")
    else {
        unreachable!("only ids for inserted entries are generated for now");
    };

    let new_state = match entry.state {
        State::Simple => unreachable!("never assigned"),
        State::Complex { .. } => State::Counter(1),
        State::Counter(c) => State::Counter(c + 1),
    };

    if let Err(e) = entry.update_state(new_state).await {
        error!("failed to update: {e}");
    }
}

fn random_id() -> u64 {
    fastrand::u64(1..ROWS_COUNT.load(Ordering::SeqCst))
}

fn ms(start: &Instant) -> f64 {
    μs(start) as f64 / 1000.0
}

fn μs(start: &Instant) -> u128 {
    start.elapsed().as_micros()
}
