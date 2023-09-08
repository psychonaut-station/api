package db

import (
	"database/sql"
	"log"
	"strings"

	_ "github.com/go-sql-driver/mysql"
)

var (
	db *sql.DB
)

type Player struct {
	Ckey           *string `json:"ckey"`
	ByondKey       *string `json:"byond_key"`
	FirstSeen      *string `json:"first_seen"`
	LastSeen       *string `json:"last_seen"`
	FirstSeenRound *int32  `json:"first_seen_round"`
	LastSeenRound  *int32  `json:"last_seen_round"`
	IP             *string `json:"ip"`
	CID            *string `json:"cid"`
	ByondAge       *string `json:"byond_age"`
}

type RoleTime struct {
	ByondKey *string `json:"byond_key"`
	Minutes  *int32  `json:"minutes"`
}

type Ban struct {
	ID             *int32  `json:"ban_id"`
	Date           *string `json:"ban_date"`
	RoundID        *int32  `json:"round_id"`
	Role           *string `json:"role"`
	ExpirationDate *string `json:"expiration_date"`
	Reason         *string `json:"reason"`
	BannedKey      *string `json:"b_ckey"`
	AdminKey       *string `json:"a_ckey"`
	Edits          *string `json:"edits"`
	UnbanDate      *string `json:"unban_date"`
	UnbanKey       *string `json:"u_ckey"`
}

func GetPlayer(ckey string) (Player, bool) {
	stmt, _ := db.Prepare(
		"SELECT ckey, byond_key, firstseen, firstseen_round_id, lastseen, lastseen_round_id, ip, computerid, accountjoindate FROM player WHERE LOWER(ckey) = ?")
	defer stmt.Close()

	row := stmt.QueryRow(strings.ToLower(ckey))

	player := Player{}
	err := row.Scan(&player.Ckey, &player.ByondKey, &player.FirstSeen, &player.FirstSeenRound,
		&player.LastSeen, &player.LastSeenRound, &player.IP, &player.CID, &player.ByondAge)
	if err != nil {
		log.Printf("DB ERR: GetPlayer, %v", err)
		return player, false
	}

	return player, true
}

func GetTopMinutes(job string) []RoleTime {
	stmt, _ := db.Prepare("SELECT ckey, minutes FROM role_time WHERE LOWER(job) = ? ORDER BY minutes DESC LIMIT 15")
	defer stmt.Close()
	rows, err := stmt.Query(strings.ToLower(job))
	if err != nil {
		return nil
	}

	result := []RoleTime{}
	data := RoleTime{}
	for rows.Next() {
		err = rows.Scan(&data.ByondKey, &data.Minutes)
		if err != nil {
			log.Printf("DB ERR: GetTopMinutes, %v", err)
			return nil
		}

		result = append(result, data)
	}

	return result
}

func GetBanByID(id int32) (Ban, bool) {
	result := Ban{}

	stmt, _ := db.Prepare(
		"SELECT id, bantime, round_id, role, expiration_time, reason, ckey, a_ckey, edits, unbanned_datetime, unbanned_ckey FROM ban WHERE id = ?")
	defer stmt.Close()

	row := stmt.QueryRow(id)
	err := row.Scan(&result.ID, &result.Date, &result.RoundID, &result.Role,
		&result.ExpirationDate, &result.Reason, &result.BannedKey,
		&result.AdminKey, &result.Edits, &result.UnbanDate,
		&result.UnbanKey)

	if err != nil {
		log.Printf("DB ERR: GetBanByID, %v", err)
		return result, false
	}

	return result, true
}

func GetBan(ckey string) ([]Ban, bool) {
	result := []Ban{}

	stmt, _ := db.Prepare(
		"SELECT id, bantime, round_id, role, expiration_time, reason, ckey, a_ckey, edits, unbanned_datetime, unbanned_ckey FROM ban WHERE LOWER(ckey) = ?")
	defer stmt.Close()

	rows, err := stmt.Query(strings.ToLower(ckey))
	for rows.Next() {
		ban := Ban{}
		err = rows.Scan(&ban.ID, &ban.Date, &ban.RoundID, &ban.Role,
			&ban.ExpirationDate, &ban.Reason, &ban.BannedKey,
			&ban.AdminKey, &ban.Edits, &ban.UnbanDate,
			&ban.UnbanKey)

		if err != nil {
			log.Printf("DB ERR: GetBan, %v", err)
			return nil, false
		}

		result = append(result, ban)
	}

	if err != nil {
		log.Printf("DB ERR: GetBanDataByID, %v", err)
		return nil, false
	}

	return result, true
}

func InitDB(dbuser string, dbpass string, dbname string) bool {
	var err error
	db, err = sql.Open("mysql", dbuser+":"+dbpass+"@/"+dbname)
	if err != nil {
		log.Fatalf("Failed to connect database, %v", err)
		return false
	}

	return true
}
