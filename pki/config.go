package pki

import "fmt"

type PKIConfig struct {
	BaseAddress string
	CAPath      string
	CSRPath     string
}

func (config *PKIConfig) caAddress() string {
	return fmt.Sprintf("%v%v", config.BaseAddress, config.CAPath)
}

func (config *PKIConfig) csrAddress() string {
	return fmt.Sprintf("%v%v", config.BaseAddress, config.CSRPath)
}
