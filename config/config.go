package config

import (
	"log"
	"os"

	"gopkg.in/yaml.v3"
)

type ServerInstance struct {
	Name              string `yaml:"Name"`
	Address           string `yaml:"Address"`
	ConnectionAddress string `yaml:"ConnectionAddress"`
	ErrorMessage      string `yaml:"ErrorMessage"`
}

type Config struct {
	Address  string `yaml:"Address"`
	Secret   string `yaml:"Secret"`
	Database struct {
		User     string `yaml:"User"`
		Password string `yaml:"Password"`
		Name     string `yaml:"Name"`
	} `yaml:"Database"`
	Servers []ServerInstance `yaml:"Servers"`
}

var (
	config Config
)

func LoadConfig(file string) bool {
	configFile, err := os.ReadFile(file)
	if err != nil {
		log.Fatalf("Config file error: %v", err)
		return false
	}

	err = yaml.Unmarshal(configFile, &config)
	if err != nil {
		log.Fatalf("Config unmarshal error: %v", err)
		return false
	}

	return true
}

func GetConfig() *Config {
	return &config
}
