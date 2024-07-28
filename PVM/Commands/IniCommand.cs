using Cocona;
using Microsoft.EntityFrameworkCore;
using PVM.Data;
using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Diagnostics;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace PVM.Commands
{
    public class IniCommand
    {
        private readonly SqliteDbContext _dbContext;

        public IniCommand(SqliteDbContext dbContext)
        {
            _dbContext = dbContext;
            _dbContext.Database.Migrate();
        }

        [Command(Description = "Open php.ini file in notepad")]
        public void Ini()
        {
            var phpVersions = _dbContext.PhpVersions.ToList();
            if (phpVersions.Count == 0)
            {
                Console.WriteLine("No versions found");
                return;
            }

            var currentVersion = _dbContext.PhpVersions.FirstOrDefault(v => v.IsCurrent);
            if (currentVersion == null)
            {
                Console.WriteLine("No current version found");
                return;
            }

            var iniPath = System.IO.Path.Join(currentVersion.Path, "php.ini");
            if (!System.IO.File.Exists(iniPath))
            {
                Console.WriteLine("php.ini not found copying php.ini-development file");
                var iniDevelopmentPath = System.IO.Path.Join(currentVersion.Path, "php.ini-development");
                if (!System.IO.File.Exists(iniDevelopmentPath))
                {
                    Console.WriteLine("php.ini-development not found");
                    return;
                }
                File.Copy(iniDevelopmentPath, iniPath);
            }

            Process.Start("notepad.exe", iniPath);
        }
    }
}
