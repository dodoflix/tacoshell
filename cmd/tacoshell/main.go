package main

import (
	"fmt"
	"io"
	"os"

	"github.com/spf13/cobra"
	"golang.org/x/crypto/ssh"
	"golang.org/x/term"
)

var (
	username string
	password string
	port     string
)

func main() {
	var rootCmd = &cobra.Command{
		Use:   "tacoshell [host]",
		Short: "A simple SSH client",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			host := args[0]
			connect(host)
		},
	}

	rootCmd.Flags().StringVarP(&username, "username", "u", "root", "SSH username")
	rootCmd.Flags().StringVarP(&password, "pass", "p", "", "SSH password")
	rootCmd.Flags().StringVarP(&port, "port", "P", "22", "SSH port")

	if err := rootCmd.Execute(); err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
}

func connect(host string) {
	config := &ssh.ClientConfig{
		User: username,
		Auth: []ssh.AuthMethod{
			ssh.Password(password),
		},
		HostKeyCallback: ssh.InsecureIgnoreHostKey(),
	}

	addr := fmt.Sprintf("%s:%s", host, port)
	client, err := ssh.Dial("tcp", addr, config)

	if err != nil {
		panic("Failed to dial: " + err.Error())
	}
	defer func(client *ssh.Client) {
		err := client.Close()
		if err != nil {
			panic("Failed to close client: " + err.Error())
		}
	}(client)

	fmt.Println("Connected to server")

	session, err := client.NewSession()
	if err != nil {
		panic("Failed to create session: " + err.Error())
	}
	defer func(session *ssh.Session) {
		err := session.Close()
		if err != nil && err != io.EOF {
			panic("Failed to close session: " + err.Error())
		}
	}(session)

	fmt.Println("Session created")

	fd := int(os.Stdin.Fd())
	w, h, _ := term.GetSize(fd)
	modes := ssh.TerminalModes{
		ssh.ECHO:          1,
		ssh.TTY_OP_ISPEED: 14400,
		ssh.TTY_OP_OSPEED: 14400,
	}
	if err := session.RequestPty("xterm-256color", h, w, modes); err != nil {
		panic("request for pty failed: " + err.Error())
	}

	fmt.Println("PTY requested")

	session.Stdout = os.Stdout
	session.Stderr = os.Stderr
	session.Stdin = os.Stdin

	if term.IsTerminal(fd) {
		oldState, err := term.MakeRaw(fd)
		if err != nil {
			panic(err)
		}
		defer func(fd int, oldState *term.State) {
			err := term.Restore(fd, oldState)
			if err != nil {
				panic(err)
			}
		}(fd, oldState)

		fmt.Println("Terminal mode set to raw")
	} else {
		fmt.Println("Not a terminal, skipping raw mode")
	}

	if err := session.Shell(); err != nil {
		panic(err)
	}

	fmt.Println("Shell started")

	err = session.Wait()
	if err != nil {
		return
	}
}
