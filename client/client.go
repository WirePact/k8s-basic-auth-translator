package client

import (
	"github.com/sirupsen/logrus"
	"k8s.io/client-go/kubernetes"
	"k8s.io/client-go/rest"
	"k8s.io/client-go/tools/clientcmd"
)

var client *kubernetes.Clientset

func GetKubernetesClient() *kubernetes.Clientset {
	if client != nil {
		return client
	}

	var err error

	inCluster, _ := rest.InClusterConfig()
	if inCluster != nil {
		logrus.Debug("Returning Kubernetes client with in cluster config.")

		client, err = kubernetes.NewForConfig(inCluster)
		if err != nil {
			logrus.WithError(err).Fatalf("The Kubernetes client could not be instantiated from inCluster config.")
		}

		return client
	}

	fileConfig, _ := clientcmd.NewDefaultClientConfigLoadingRules().Load()
	clientConfig, _ := clientcmd.NewDefaultClientConfig(*fileConfig, nil).ClientConfig()
	client, err = kubernetes.NewForConfig(clientConfig)
	if err != nil {
		logrus.WithError(err).Fatalf("The Kubernetes client could not be instantiated from inCluster config.")
	}

	return client
}
