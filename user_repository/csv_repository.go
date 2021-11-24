package user_repository

import (
	"encoding/csv"
	"io"
	"os"

	"github.com/sirupsen/logrus"
)

type csvRepository struct {
	entries []csvUserEntry
}

type csvUserEntry struct {
	userID   string
	username string
	password string
}

func newCSVRepository(csvPath string) UserRepository {
	file, err := os.Open(csvPath)
	if err != nil {
		logrus.WithError(err).Fatalf("Could not load csv file from path '%v'.", csvPath)
	}
	defer file.Close()

	csvReader := csv.NewReader(file)
	var records []csvUserEntry
	firstRow := true

	for {
		record, err := csvReader.Read()
		if firstRow {
			firstRow = false
			continue
		}

		if err == io.EOF {
			break
		}
		if err != nil {
			logrus.WithError(err).Errorln("Could not read record from CSV.")
		}

		records = append(records, csvUserEntry{
			userID:   record[0],
			username: record[1],
			password: record[2],
		})
	}

	return &csvRepository{records}
}

func (repository *csvRepository) LookupUserID(username string, password string) *string {
	for _, entry := range repository.entries {
		if entry.username == username && entry.password == password {
			return &entry.userID
		}
	}

	return nil
}
