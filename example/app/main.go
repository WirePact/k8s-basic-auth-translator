package main

import (
	"flag"
	"fmt"
	"io/ioutil"
	"net/http"

	"github.com/gin-gonic/gin"
	"github.com/sirupsen/logrus"
)

type album struct {
	ID     string `json:"id"`
	Title  string `json:"title"`
	Artist string `json:"artist"`
}

var (
	port = flag.Int("port", 8001, "Port for the webserver.")
	api  = flag.String("api", "http://localhost:8000/albums", "Address for the api call.")
)

const (
	username = "user"
	password = "pass"
)

func main() {
	flag.Parse()

	logrus.Infof("Starting webserver on port ':%v'", *port)
	logrus.Infof("Will call API on '%v'", *api)

	router := gin.Default()
	router.LoadHTMLGlob("templates/*")

	var data string

	router.GET("/", func(context *gin.Context) {
		context.HTML(http.StatusOK, "index.html", gin.H{
			"data": data,
		})
		data = ""
	})

	router.POST("/api-call", func(context *gin.Context) {
		request, _ := http.NewRequest("GET", *api, nil)
		request.SetBasicAuth(username, password)
		client := &http.Client{}
		response, err := client.Do(request)
		if err != nil {
			logrus.WithError(err).Errorln("Error connecting to API.")
			data = err.Error()
		} else {
			bodyText, _ := ioutil.ReadAll(response.Body)
			logrus.Infof("Fetched from API: %v", string(bodyText))
			data = string(bodyText)
		}

		context.Redirect(http.StatusFound, "/")
	})

	err := router.Run(fmt.Sprintf(":%v", *port))
	if err != nil {
		logrus.WithError(err).Fatal("Could not start server.")
	}
}
