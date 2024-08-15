# PVM(Php Version Management Tool) For windows only

## Description

This project is aimed at simplifying the management of PHP versions on your system using a tool called PHP Version Manager (PVM). PVM allows you to easily install, switch between, and manage different PHP versions on your machine.

## Upcoming Features

1. **Install PHP Extensions from Terminal**: Install PHP extensions directly from the terminal, eliminating the process of copying and pasting and making changes to ini file.

2. **Configure PHP Settings from Terminal**: Access and modify various PHP configurations directly from the terminal, without needing to edit PHP configuration files manually.

## Setup

1. **Extract Application**: Begin by extracting the PVM application to a directory of your choice on your system.

2. **Set Environment Variables**: Add the `<application path>` and `<application path>/php` directories to your environment path to make the PVM commands accessible from anywhere in your terminal.

## Usage

Below are some common commands you can use with PVM:

- **Install PHP**: Use the following commands to install PHP:
  ```
  pvm install 8.3.3
  pvm install 8.3.3 --type ts
  ```

- **Change PHP Version**: Switch between installed PHP versions using:
  ```
  pvm use 8.3.3
  ```

- **Open Extension Folder**: Open the PHP extension folder:
  ```
  pvm ext
  ```

- **Enable extension**: enable current activated php extension:
  ```
  pvm ext-enable curl
  ```

- **Open INI File in Notepad**: Open the PHP configuration (INI) file in Notepad:
  ```
  pvm ini
  ```

- **Show Available PHP Versions**: List all available PHP versions:
  ```
  pvm list
  ```

- **Add PHP Version to PVM**: Add the current local PHP installation to PVM:
  ```
  pvm add --path <path of php> --version <version>
  ```

## Notes

- Ensure that you have the necessary permissions to modify environment variables and access system directories before performing setup steps.
- For advanced usage and additional commands, refer to the PVM documentation or run `pvm --help` in your terminal.

## License

This project is licensed under the [MIT License](LICENSE).
