package user_repository

import "github.com/sirupsen/logrus"

type UserRepository interface {
	// LookupUserID returns the (possible) userID of a user/password combination.
	// If no combination is found, nil is returned.
	LookupUserID(username string, password string) string
	LookupUsernameAndPassword(userID string) (string, string)
}

var repository UserRepository

func GetUserRepository() UserRepository {
	return repository
}

// ConfigureCSVRepository sets the user repository (for user id lookups)
// to a csv file.
func ConfigureCSVRepository(csvPath string) {
	if repository != nil {
		logrus.Fatalln("Repository is already configured. Cannot configure CSV repository.")
	}

	repository = newCSVRepository(csvPath)
}
