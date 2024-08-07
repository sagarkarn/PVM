﻿using Cocona;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace PVM.Commands
{
    public class ExtEnableCommand
    {
        [Command(Description = "Enable extension that is alreay installed in current php version")]
        public void ExtEnable([Argument]string ext)
        {
            if (!DllInExtDir(ext))
            {
                return;
            }
            

            var phpIniPath = Path.Join(Directory.GetCurrentDirectory(), "php", "php.ini");
            if (!File.Exists(phpIniPath))
            {
                Console.WriteLine("php.ini not found copying php.ini-development file");
                var phpIniDevelopmentPath = Path.Join(Directory.GetCurrentDirectory(), "php", "php.ini-development");
                if (!File.Exists(phpIniDevelopmentPath))
                {
                    Console.WriteLine("php.ini-development not found");
                    return;
                }
                File.Copy(phpIniDevelopmentPath, phpIniPath);
            }

            var lines = File.ReadAllLines(phpIniPath).ToList();
            var extLine = lines.FirstOrDefault(l => l.StartsWith("extension=" + ext));
            if (extLine != null)
            {
                Console.WriteLine("extension already enabled");
                return;
            }
            extLine = lines.FirstOrDefault(l => l.StartsWith(";extension=" + ext));
            if (extLine == null)
            {
                Console.WriteLine("extension not found in ini file");
            }
            else
            {
                var index = lines.IndexOf(extLine);
                lines[index] = "extension=" + ext;
                File.WriteAllLines(phpIniPath, lines);
                Console.WriteLine("extension enabled");
            }
        }
        [Ignore]
        private static bool DllInExtDir(string ext)
        {
            var dllName = "php_" + ext + ".dll";
            var extPath = Path.Join(Directory.GetCurrentDirectory(), "php", "ext");
            if (!Directory.Exists(extPath))
            {
                Console.WriteLine("ext not found");
                return false;
            }
            var extDll = Path.Join(extPath, dllName);
            if (!File.Exists(extDll))
            {
                Console.WriteLine("file not found");
                return false;
            }
            return true;
        }
    }
}
