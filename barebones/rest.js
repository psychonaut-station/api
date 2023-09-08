const SHUTTLE_IDLE = "idle"
const SHUTTLE_IGNITING = "igniting"
const SHUTTLE_RECALL = "recalled"
const SHUTTLE_CALL = "called"
const SHUTTLE_DOCKED = "docked"
const SHUTTLE_STRANDED = "stranded"
const SHUTTLE_DISABLED = "disabled"
const SHUTTLE_ESCAPE = "escape"
const SHUTTLE_ENDGAME = "endgame: game over"
const SHUTTLE_RECHARGING = "recharging"
const SHUTTLE_PREARRIVAL = "landing"

function GetShuttleMode(mode) {
    switch (mode) {
		case SHUTTLE_IGNITING:
			return "IGN";
		case SHUTTLE_RECALL:
			return "RCL";
		case SHUTTLE_CALL:
			return "ETA";
		case SHUTTLE_DOCKED:
			return "ETD";
		case SHUTTLE_ESCAPE:
			return "ESC";
		case SHUTTLE_STRANDED:
			return "ERR";
		case SHUTTLE_RECHARGING:
			return "RCH";
		case SHUTTLE_PREARRIVAL:
			return "LDN";
		case SHUTTLE_DISABLED:
			return "DIS";
        case "endgame: game over":
            return "END";
        default:
            return "";
    }
}

/// Game is loading
const GAME_STATE_STARTUP = 0
/// Game is loaded and in pregame lobby
const GAME_STATE_PREGAME = 1
/// Game is attempting to start the round
const GAME_STATE_SETTING_UP = 2
/// Game has round in progress
const GAME_STATE_PLAYING = 3
/// Game has round finished
const GAME_STATE_FINISHED = 4

function GetGameState(state) {
    switch (state) {
		case GAME_STATE_STARTUP:
			return "Yükleniyor";
		case GAME_STATE_PREGAME:
			return "Lobi";
		case GAME_STATE_SETTING_UP:
			return "Başlıyor";
		case GAME_STATE_PLAYING:
			return "Devam ediyor";
		case GAME_STATE_FINISHED:
			return "Bitti";
        default:
            return "";
    }
}

function pad(n, width, z) {
    z = z || '0';
    n = n + '';
    return n.length >= width ? n : new Array(width - n.length + 1).join(z) + n;
}

function secondsToTime(seconds) {
    var output = '';
    if (seconds >= 86400)
        output += Math.floor(seconds / 86400) + ':';
    if (seconds >= 3600)
        output += pad(Math.floor(seconds / 3600) % 24, 2) + ':';
    output += pad(Math.floor((seconds / 60) % 60), 2) + ':' + pad(Math.floor(seconds) % 60, 2);
    return output;
}

function GetShuttleTime(time_left, mode) {
    if(mode == SHUTTLE_STRANDED || mode == SHUTTLE_DISABLED)
        return "--:--"

    return secondsToTime(time_left);
}

function SendHeartbeat() {
    var api = 'https://api.psychonautstation.com/v1/server';
    const xhr = new XMLHttpRequest();
    xhr.onload = () => { StatusHandler(xhr.status, xhr.response) };
    xhr.open("GET", api, true);
    xhr.setRequestHeader(
        "Content-Type",
        "application/x-www-form-urlencoded",
    );
    xhr.send();
}

function StatusHandler(status, response) {
    data = document.getElementById("status");

    if (status === 200) {
        const header = JSON.parse(response);
        const server_data = header.response;
        var htmlData = "";

        server_data.forEach(element => {
            htmlData += "<p style='font-size:30px'>"+element.name+"</p><br>"
            if (element.server_status == 0) {
                htmlData += "<p style='font-size:20px'>Bağlantı hatası: "+element.err_str+"</p><br>";
                return;
            }

            htmlData += "<p>"+element.players+" - "+GetGameState(element.gamestate)+" - "+element.map+"</p><br>"
            htmlData += "<p>"+element.security_level+" security level</p><br>"
            htmlData += "<p>"+secondsToTime(element.round_duration);
            if (element.shuttle_mode != SHUTTLE_IDLE)
                htmlData += " - " + GetShuttleMode(element.shuttle_mode) + " " + secondsToTime(element.shuttle_time);

            htmlData += "</p><br>";
        });

        data.innerHTML = htmlData;
    } else {
        data.innerHTML = "Internal API error";
    }
}