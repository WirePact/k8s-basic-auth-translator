package main

import (
	"flag"
	"fmt"

	"github.com/gin-gonic/gin"
	"github.com/sirupsen/logrus"
)

type album struct {
	ID     string `json:"id"`
	Title  string `json:"title"`
	Artist string `json:"artist"`
}

var (
	port = flag.Int("port", 8000, "Port for the webserver.")
)

var data = [...]album{
	{ID: "one", Title: "39 Seconds", Artist: "Marcus Warner"},
	{ID: "two", Title: "Born To Lose", Artist: "TLMT"},
	{ID: "three", Title: "Midgard (Tour Edition)", Artist: "Faun"},
}

func getAlbums(context *gin.Context) {
	context.JSON(200, data)
}

func main() {
	flag.Parse()

	logrus.Infof("Starting webserver on port ':%v'", *port)

	router := gin.Default()

	secure := router.Group("/", gin.BasicAuth(gin.Accounts{
		"user": "pass",
	}))
	secure.GET("albums", getAlbums)

	err := router.Run(fmt.Sprintf(":%v", *port))
	if err != nil {
		logrus.WithError(err).Fatal("Could not start server.")
	}
}
