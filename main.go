package main

import (
	"encoding/base64"
	"os"
	"strings"

	"github.com/WirePact/go-translator"
	"github.com/WirePact/go-translator/translator"
	"github.com/WirePact/go-translator/wirepact"
	core "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
	auth "github.com/envoyproxy/go-control-plane/envoy/service/auth/v3"
	"github.com/sirupsen/logrus"
	"wirepact.ch/k8s-basic-auth-translator/user_repository"
)

func main() {
	logrus.SetLevel(logrus.InfoLevel)

	if os.Getenv("CSV_PATH") != "" {
		user_repository.ConfigureCSVRepository(os.Getenv("CSV_PATH"))
	}
	if os.Getenv("KUBERNETES_SECRET") != "" {
		user_repository.ConfigureKubernetesRepository(os.Getenv("KUBERNETES_SECRET"))
	}

	config, err := go_translator.NewConfigFromEnvironmentVariables(ingress, egress)
	if err != nil {
		logrus.WithError(err).Fatalln("Could not initialize translator config.")
	}
	server, err := go_translator.NewTranslator(&config)
	if err != nil {
		logrus.WithError(err).Fatalln("Could not create translator.")
	}

	server.Start()
}

func ingress(subject string, req *auth.CheckRequest) (translator.IngressResult, error) {
	logger := logrus.
		WithFields(logrus.Fields{
			"type":       "ingress",
			"request_id": req.Attributes.Request.Http.Id,
			"host":       req.Attributes.Request.Http.Host,
			"path":       req.Attributes.Request.Http.Path,
			"method":     req.Attributes.Request.Http.Method,
		})
	logger.Traceln("Checking ingress request.")

	repository := user_repository.GetUserRepository()
	username, password := repository.LookupUsernameAndPassword(subject)

	if username == "" || password == "" {
		logger.Infof("Could not find username/password for subject '%v'.", subject)
		return translator.IngressResult{
			Forbidden: "Could not find username/password for subject.",
		}, nil
	}

	logger.Infof("Userinformation found for user '%v'. Return OK with Basic Auth credentials.", subject)
	return translator.IngressResult{
		HeadersToAdd: []*core.HeaderValue{
			{
				Key:   wirepact.AuthorizationHeader,
				Value: "Basic " + base64.StdEncoding.EncodeToString([]byte(username+":"+password)),
			},
		},
	}, nil
}

func egress(req *auth.CheckRequest) (translator.EgressResult, error) {
	logger := logrus.
		WithFields(logrus.Fields{
			"type":       "egress",
			"request_id": req.Attributes.Request.Http.Id,
			"host":       req.Attributes.Request.Http.Host,
			"path":       req.Attributes.Request.Http.Path,
			"method":     req.Attributes.Request.Http.Method,
		})
	logger.Traceln("Checking egress request.")

	header, ok := req.Attributes.Request.Http.Headers[wirepact.AuthorizationHeader]
	if !ok {
		logger.Debugln("The request has no authorization header. Skipping.")
		return translator.EgressResult{Skip: true}, nil
	} else if !strings.Contains(header, "Basic") {
		logger.Debugln("The request is not Basic Auth authorized. Skipping.")
		return translator.EgressResult{Skip: true}, nil
	}

	logger.Debugln("The request contains a Basic Auth signature. Convert to WirePact JWT.")

	payload, _ := base64.StdEncoding.DecodeString(strings.ReplaceAll(header, "Basic ", ""))
	authPair := strings.SplitN(string(payload), ":", 2)

	if len(authPair) != 2 {
		logger.Warnf("The Basic Auth data was corruped. Received '%v'. Not Authorizing Request.", string(payload))
		return translator.EgressResult{Forbidden: "The Basic Auth data was corrupted."}, nil
	}

	username := authPair[0]
	password := authPair[1]
	repository := user_repository.GetUserRepository()

	return translator.EgressResult{
		UserID:          repository.LookupUserID(username, password),
		HeadersToRemove: []string{wirepact.AuthorizationHeader},
	}, nil
}
