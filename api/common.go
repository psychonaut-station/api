package api

import (
	"api/config"
	"encoding/json"
	"net/http"
	"net/url"
	"reflect"
	"strconv"
	"time"
)

const ResponseFail = 0
const ResponseSuccess = 1
const ResponseDenied = 2
const ResponseBadAuth = 3

// Generic response header
type GenericHeader[T interface{}] struct {
	Status int    `json:"status"`
	Reason string `json:"reason"`
	Body   T      `json:"response"`
}

type CachedResponse[T interface{}] struct {
	Cooldown   time.Duration
	LastAccess time.Time
	Var        T
}

func ShouldRefreshResponse[T interface{}](response CachedResponse[T]) bool {
	return time.Since(response.LastAccess) >= response.Cooldown
}

func (w *CachedResponse[T]) Init(cooldown time.Duration) {
	w.Cooldown = cooldown
	w.LastAccess = time.Time{}
}

func Confidential(w http.ResponseWriter, r *http.Request) bool {
	token := r.Header.Get("token")
	if token != config.GetConfig().Secret {
		WriteHeaderResponse(w, ResponseBadAuth, 0)
		return false
	}

	return true
}

func WriteHeaderResponseCustom[T interface{}](w http.ResponseWriter, status int, reason string, body T) {
	header := GenericHeader[T]{Status: status, Reason: reason, Body: body}
	WriteJson(w, header)
}

func WriteHeaderResponse[T interface{}](w http.ResponseWriter, status int, body T) {
	switch status {
	case ResponseFail:
		WriteHeaderResponseCustom[int](w, status, "fail", 0)
		break
	case ResponseSuccess:
		WriteHeaderResponseCustom[T](w, status, "success", body)
		break
	case ResponseDenied:
		WriteHeaderResponseCustom[int](w, status, "denied", 0)
		break
	case ResponseBadAuth:
		WriteHeaderResponseCustom[int](w, status, "bad auth", 0)
		break
	}
}

func url_to_int(u url.Values, v string) int32 {
	val, _ := strconv.Atoi(u.Get(v))
	return int32(val)
}

func url_to_f32(u url.Values, v string) float32 {
	val, _ := strconv.ParseFloat(u.Get(v), 32)
	return float32(val)
}

func to_int(v string) int32 {
	val, _ := strconv.Atoi(v)
	return int32(val)
}

func to_f32(v string) float32 {
	val, _ := strconv.ParseFloat(v, 32)
	return float32(val)
}

func WriteJson(w http.ResponseWriter, v any) {
	json.NewEncoder(w).Encode(v)
}

func AssignStruct[T interface{}](src any) T {
	var dst T
	typeSrc := reflect.TypeOf(src)
	valSrc := reflect.ValueOf(src)
	valDst := reflect.ValueOf(&dst).Elem()

	for i := 0; i < typeSrc.NumField(); i++ {
		srcTypeField := typeSrc.Field(i)
		export := srcTypeField.Tag.Get("export")
		dstField := valDst.FieldByName(export)
		if dstField.IsValid() {
			dstField.Set(valSrc.Field(i))
		}
	}

	return dst
}
