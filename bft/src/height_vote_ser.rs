// 投票信息
#[derive(Serialize, Deserialize, Debug, Clone)]
struct RoundVotes {
    Round: i32,
    Prevotes: Vec<String>,
    PrevotesBitArray: String,
    Precommits: Vec<String>,
    PrecommitsBitArray: String,
}