package main

import (
	"api/api"
	"api/config"
	"api/db"
	"log"
	"net/http"
	"os"
	"time"

	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	"github.com/go-chi/cors"
	"github.com/go-chi/render"
)

func main() {
	config.LoadConfig("config/config.api.yaml")

	os.Mkdir("data/logs", os.ModePerm)
	logFile, err := os.Create("api.log")
	if err != nil {
		log.Fatalf("Failed to create api log file, %v.", err)
		return
	}

	logger := log.New(logFile, "", log.LstdFlags)
	middleware.DefaultLogger = middleware.RequestLogger(&middleware.DefaultLogFormatter{Logger: logger, NoColor: true})

	r := chi.NewRouter()
	r.Use(middleware.RequestID)
	r.Use(middleware.Logger)
	r.Use(middleware.Recoverer)
	r.Use(middleware.URLFormat)
	r.Use(cors.Handler(cors.Options{
		AllowedOrigins:   []string{"https://*", "http://*"},
		AllowedMethods:   []string{"GET", "POST"},
		AllowOriginFunc:  func(r *http.Request, origin string) bool { return true },
		AllowedHeaders:   []string{"Accept", "Authorization", "Content-Type", "X-CSRF-Token"},
		ExposedHeaders:   []string{"Link"},
		AllowCredentials: false,
		MaxAge:           300,
	}))
	r.Use(render.SetContentType(render.ContentTypeJSON))
	r.Use(middleware.Timeout(60 * time.Second))

	var cfg = config.GetConfig()
	if !db.InitDB(cfg.Database.User, cfg.Database.Password, cfg.Database.Name) {
		return
	}

	log.Println("Initializing V1")
	api.InitV1(r)
	log.Printf("Listening to %v", cfg.Address)
	http.ListenAndServe(cfg.Address, r)
}
