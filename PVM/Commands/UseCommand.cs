using Cocona;
using Microsoft.EntityFrameworkCore;
using PVM.Data;
using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Linq;
using System.Reflection;
using System.Text;
using System.Threading.Tasks;

namespace PVM.Commands
{
    public class UseCommand
    {
        private readonly SqliteDbContext _dbContext;

        public UseCommand(SqliteDbContext dbContext)
        {
            _dbContext = dbContext;
            _dbContext.Database.Migrate();
        }

        [Command(Description = "Switch to use the specified version")]
        public void Use([Argument]string version)
        {
            var phpVersion = _dbContext.PhpVersions.OrderByDescending(x=>x.Version). FirstOrDefault(v => v.Version.StartsWith(version));
            if (phpVersion == null)
            {
                Console.WriteLine($"Version {version} not found");
                return;
            }

            if(phpVersion.IsCurrent)
            {
                Console.WriteLine($"Version {version} is already in use");
                return;
            }

            var path = phpVersion.Path;
            Console.WriteLine($"Using version {phpVersion.Path}");
            if (!Directory.Exists(path))
            {
                Console.WriteLine($"Path {path} not found");
                return;
            }


            var currentVersion = _dbContext.PhpVersions.FirstOrDefault(v => v.IsCurrent);

            if (currentVersion != null)
            {
                var movedPath = Path.Join(System.AppContext.BaseDirectory, "php" + currentVersion.Version);
                Console.WriteLine($"Moving {currentVersion.Path} to {movedPath}");
                Directory.Move(currentVersion.Path, movedPath);
                Console.WriteLine($"Moved {currentVersion.Path} to {movedPath}");
                currentVersion.IsCurrent = false;
                currentVersion.Path = movedPath;
            }

            Directory.Move(phpVersion.Path, Path.Join(System.AppContext.BaseDirectory, "php"));
            phpVersion.Path = Path.Join(System.AppContext.BaseDirectory, "php");
            phpVersion.IsCurrent = true;
            _dbContext.SaveChanges();
            Console.WriteLine($"Using version {version}");
        }
    }
}
