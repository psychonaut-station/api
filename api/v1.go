package api

import (
	"api/byond"
	"api/config"
	"api/db"
	"net/http"
	"net/url"
	"time"

	"github.com/go-chi/chi/v5"
)

// Direct representation of status topic
type ServerStatus struct {
	Name             string  `json:"name"`
	ServerStatus     int32   `json:"server_status"`
	ErrorMessage     string  `json:"err_str"`
	RoundID          int32   `json:"round_id"`               // ID of the current round
	Hub              bool    `json:"hub"`                    // Is server visible to hub
	Players          int32   `json:"players"`                // Player count
	Admins           int32   `json:"admins"`                 // Visible admin count
	Map              string  `json:"map"`                    // Map name
	SecurityLevel    string  `json:"security_level"`         // Security level in text
	RoundDuration    int32   `json:"round_duration"`         // Integer round duration
	GameState        int32   `json:"gamestate"`              // SSticker's game states, GAME_STATE_*
	TimeDilation     float32 `json:"time_dilation_current"`  // Current timedilation
	TimeDilationAvg  float32 `json:"time_dilation_avg"`      // Average timedilation
	TimeDilationSlow float32 `json:"time_dilation_avg_slow"` // Slow average timedilation
	TimeDilationFast float32 `json:"time_dilation_avg_fast"` // Fast average timedilation
	ShuttleMode      string  `json:"shuttle_mode"`           // Current state of shuttle in string
	ShuttleTime      int32   `json:"shuttle_time"`           // Integer timer
	ConnectionInfo   string  `json:"connection_info"`        // ip:port
}

type WebhookResponse struct {
	Hook struct {
		Config struct {
			ContentType string `json:"content_type"`
			Secret      string `json:"secret"`
		} `json:"config"`
	} `json:"hook"`
	Repository struct {
		URL string `json:"html_url"`
	} `json:"repository"`
}

var (
	serverResponseW      CachedResponse[[]ServerStatus]
	playTimeLeaderboardW CachedResponse[[]db.RoleTime]
)

func StatusCb(w http.ResponseWriter, r *http.Request) {
	if ShouldRefreshResponse(serverResponseW) {
		serverResponseW.LastAccess = time.Now()
		serverResponseW.Var = nil

		var cfg = config.GetConfig()
		for _, server := range cfg.Servers {
			err, dataType, data := byond.Topic(server.Address, "?status")

			// Base server info
			status := ServerStatus{}
			status.Name = server.Name
			status.ServerStatus = 0
			status.ErrorMessage = server.ErrorMessage
			status.ConnectionInfo = server.ConnectionAddress

			if err != nil || dataType != byond.TopicTypeString {
				serverResponseW.Var = append(serverResponseW.Var, status)
				continue
			}

			// Online server info
			u, _ := url.ParseQuery(string(data))

			status.ServerStatus = 1
			status.RoundID = url_to_int(u, "round_id")
			status.Hub = url_to_int(u, "hub") == 1
			status.Players = url_to_int(u, "players")
			status.Admins = url_to_int(u, "admins")
			status.Map = u.Get("map_name")
			status.SecurityLevel = u.Get("security_level")
			status.RoundDuration = url_to_int(u, "round_duration")
			status.GameState = url_to_int(u, "gamestate")
			status.TimeDilation = url_to_f32(u, "time_dilation_current")
			status.TimeDilationAvg = url_to_f32(u, "time_dilation_avg")
			status.TimeDilationSlow = url_to_f32(u, "time_dilation_avg_slow")
			status.TimeDilationFast = url_to_f32(u, "time_dilation_avg_fast")
			status.ShuttleMode = u.Get("shuttle_mode")
			status.ShuttleTime = url_to_int(u, "shuttle_time")

			serverResponseW.Var = append(serverResponseW.Var, status)
		}
	}

	WriteHeaderResponse(w, ResponseSuccess, serverResponseW.Var)
}

func PlayerCb(w http.ResponseWriter, r *http.Request) {
	if !Confidential(w, r) {
		return
	}

	ckey := r.URL.Query().Get("ckey")
	result, success := db.GetPlayer(ckey)

	if success {
		WriteHeaderResponse(w, ResponseSuccess, result)
	} else {
		WriteHeaderResponseCustom(w, ResponseFail, "Player not found", 0)
	}
}

func PlayerLeaderboardCb(w http.ResponseWriter, r *http.Request) {
	job := r.URL.Query().Get("job")
	if len(job) > 0 {
		if !Confidential(w, r) {
			return
		}
		data := db.GetTopMinutes(job)
		playTimes := []db.RoleTime{}
		for _, v := range data {
			playTimes = append(playTimes, db.RoleTime{ByondKey: v.ByondKey, Minutes: v.Minutes})
		}

		WriteHeaderResponse(w, ResponseSuccess, playTimes)
		return
	}

	if ShouldRefreshResponse(playTimeLeaderboardW) {
		playTimeLeaderboardW.LastAccess = time.Now()
		playTimeLeaderboardW.Var = nil

		data := db.GetTopMinutes("Living")
		for _, v := range data {
			playTimeLeaderboardW.Var = append(playTimeLeaderboardW.Var,
				db.RoleTime{ByondKey: v.ByondKey, Minutes: v.Minutes})
		}
	}

	WriteHeaderResponse(w, ResponseSuccess, playTimeLeaderboardW.Var)
}

func BanCheckCb(w http.ResponseWriter, r *http.Request) {
	if !Confidential(w, r) {
		return
	}

	q := r.URL.Query()
	status := ResponseFail

	if q.Has("id") {
		banID := url_to_int(q, "id")

		ban, success := db.GetBanByID(banID)
		if success {
			status = ResponseSuccess
		}

		WriteHeaderResponse(w, status, ban)
	} else if q.Has("ckey") {
		ckey := q.Get("ckey")

		ban, success := db.GetBan(ckey)
		if success {
			status = ResponseSuccess
		}

		WriteHeaderResponse(w, status, ban)
	}
}

func InitV1(r chi.Router) {
	serverResponseW.Init(5 * time.Second)
	playTimeLeaderboardW.Init(1 * time.Hour)

	r.Route("/v1", func(r chi.Router) {
		r.Route("/server", func(r chi.Router) {
			r.Get("/", StatusCb)
		})

		r.Route("/player", func(r chi.Router) {
			r.Get("/", PlayerCb)
			r.Get("/top", PlayerLeaderboardCb)
			r.Get("/ban", BanCheckCb)
		})
	})
}
