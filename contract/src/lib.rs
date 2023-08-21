/*
 * Example smart contract written in RUST
 *
 * Learn more about writing NEAR smart contracts with Rust:
 * https://near-docs.io/develop/Contract
 *
 */

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, Promise};

use std::collections::HashMap;

type RoomId = String;
type CheckInDate = String;

#[derive(Serialize, Deserialize, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum UsageStatus {
    Available,                           // 空き
     Stay { check_in_date: CheckInDate }, // 掲出中
}
 
 // ショップオーナーが登録した一覧を表示する際に使用
#[derive(Serialize, Deserialize, Debug, BorshSerialize, BorshDeserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct RegisteredRoom {
    name: String,
    image: String,
    beds: u8,
    description: String,
    location: String,
    price: U128,
    status: UsageStatus,
 }
 
 // 実際にブロックチェーン上に保存されるショップのデータ
 #[derive(BorshDeserialize, BorshSerialize)]
 pub struct Room {
    name: String,        // ショップの名前
    owner_id: AccountId, // オーナーのアカウントID
    image: String,       // ショップの画像（URL）
    beds: u8,            // 掲出場所（ベッド）の数
    description: String, // 部屋の説明
    location: String,    // 施設の場所
    price: U128,         // 掲出料
    status: UsageStatus, // 利用状況
    booked_info: HashMap<CheckInDate, AccountId>, // 予約データ[掲出日, 掲出者のアカウントID]
 }

 #[near_bindgen]
 #[derive(BorshSerialize, BorshDeserialize)]
 pub struct Contract {
     rooms_per_owner: LookupMap<AccountId, Vec<RoomId>>,
     rooms_by_id: HashMap<RoomId, Room>,
}

impl Default for Contract {
     fn default() -> Self {
         Self {
            rooms_per_owner: LookupMap::new(b"m"),
            rooms_by_id: HashMap::new(),
         }
     }
 }

 #[near_bindgen]
 impl Contract {
     pub fn add_room_to_owner(
         &mut self,
         name: String,
         image: String,
         beds: u8,
         description: String,
         location: String,
         price: U128,
     ) 
     {
         // 関数をコールしたアカウントIDを取得
         let owner_id = env::signer_account_id();
 
         // 部屋のIDをオーナーのアカウントIDと部屋の名前で生成
         let room_id = format!("{}{}", owner_id, name);
 
         // Room構造体を、データを入れて生成
         let new_room = Room {
             owner_id: owner_id.clone(),
             name,
             image,
             beds,
             description,
             location,
             price,
             status: UsageStatus::Available,
             booked_info: HashMap::new(),
         };
 
         // 部屋のデータを`RoomId`と紐付けて保存
         self.rooms_by_id.insert(room_id.clone(), new_room);
 
         // オーナーのアカウントIDと`RoomId`のVectorを紐付けて保存
         match self.rooms_per_owner.get(&owner_id) {
             // オーナーが既に別の部屋を登録済みの時
             Some(mut rooms) => {
                 rooms.push(room_id);
                 self.rooms_per_owner.insert(&owner_id, &rooms);
             }
             // オーナーが初めて部屋を登録する時
             None => {
                 // `room_id`を初期値にVectorを生成する
                 let new_rooms = vec![room_id];
                 self.rooms_per_owner.insert(&owner_id, &new_rooms);
             }
         }
     }
     // `room_id`が既に存在するかを確認する
     // 同じ部屋名を複数所有することは想定しないため、`add_room_to_owner`を実行する前にコールされる
     pub fn exists(&self, owner_id: AccountId, room_name: String) -> bool {
         let room_id = format!("{}{}", owner_id, room_name);

         self.rooms_by_id.contains_key(&room_id)
     }

     pub fn get_rooms_registered_by_owner(&self, owner_id: AccountId) -> Vec<RegisteredRoom> {
     // 空のVectorを生成する
     let mut registered_rooms = vec![];
 
     match self.rooms_per_owner.get(&owner_id) {
         // オーナーが部屋のデータを保存していた時
         Some(rooms) => {
             // 保存されている全ての部屋のデータに対し、一つずつ処理を行う
             for room_id in rooms {
                 // `room_id`をkeyとして、マップされている`Room`構造体のデータを取得
                 let room = self.rooms_by_id.get(&room_id).expect("ERR_NOT_FOUND_ROOM");
 
                 // 部屋のステータスを複製する
                 let status = match &room.status {
                 // ステータスが`Available`の時
                        UsageStatus::Available => UsageStatus::Available,
                        // ステータスが`Stay`の時
                        UsageStatus::Stay { check_in_date } => UsageStatus::Stay {
                            check_in_date: check_in_date.clone(),
                        },
                    };
 
                 // 取得した部屋のデータをもとに、`RegisteredRoom`構造体を生成
                 let registered_room = RegisteredRoom {
                     name: room.name.clone(),
                     beds: room.beds,
                     image: room.image.clone(),
                     description: room.description.clone(),
                     location: room.location.clone(),
                     price: room.price,
                     status,
                 };
                 // Vectorに追加
                 registered_rooms.push(registered_room);
             }
             registered_rooms
         }         // 部屋のデータが存在しない時
         None => registered_rooms,
     }
 }

 }