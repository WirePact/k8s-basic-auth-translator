package user_repository

import (
	"context"
	"encoding/base64"
	"strings"

	"github.com/sirupsen/logrus"
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	"wirepact.ch/k8s-basic-auth-translator/kubernetes"
)

type kubernetesRepository struct {
	secretName string
}

type kubernetesSecretEntry struct {
	username string
	password string
}

func newKubernetesRepository(secretName string) UserRepository {
	return &kubernetesRepository{secretName: secretName}
}

func (repository *kubernetesRepository) LookupUserID(username string, password string) string {
	entries, err := repository.getSecretEntries()
	if err != nil {
		logrus.WithError(err).Errorf("Could not fetch secret data.")
		return ""
	}

	for id, data := range entries {
		if data.username == username && data.password == password {
			return id
		}
	}

	return ""
}

func (repository *kubernetesRepository) LookupUsernameAndPassword(userID string) (string, string) {
	entries, err := repository.getSecretEntries()
	if err != nil {
		logrus.WithError(err).Errorf("Could not fetch secret data.")
		return "", ""
	}

	for id, data := range entries {
		if id == userID {
			return data.username, data.password
		}
	}

	return "", ""
}

func (repository *kubernetesRepository) getSecretEntries() (map[string]kubernetesSecretEntry, error) {
	client := kubernetes.GetKubernetesClient()

	secret, err := client.CoreV1().Secrets(kubernetes.GetCurrentNamespace()).Get(context.TODO(), repository.secretName, metav1.GetOptions{})
	if err != nil {
		logrus.WithError(err).Errorf("Error during retrieval of secret.")
		return nil, err
	}

	entries := make(map[string]kubernetesSecretEntry)

	for id, data := range secret.Data {
		decoded, err := base64.StdEncoding.DecodeString(string(data))
		if err != nil {
			logrus.WithError(err).Errorf("Error during decoding secret data, skipping %v.", id)
			continue
		}

		userdata := strings.SplitN(string(decoded), ":", 2)
		entries[id] = kubernetesSecretEntry{
			username: userdata[0],
			password: userdata[1],
		}
	}

	return entries, nil
}
