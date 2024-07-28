using Cocona;
using Microsoft.EntityFrameworkCore;
using PVM.Data;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace PVM.Commands
{
    public class AddCommand
    {
        private readonly SqliteDbContext _dbContext;

        public AddCommand(SqliteDbContext dbContext)
        {
            _dbContext = dbContext;
            _dbContext.Database.Migrate();
        }
        [Command("Let manage pvm add already installed php.")]
        public void Add(string version, string path)
        {
            var phpVersion = _dbContext.PhpVersions.FirstOrDefault(v => v.Version == version);
            if (phpVersion != null)
            {
                Console.WriteLine($"Version {version} already exists");
                return;
            }

            var movedPath = Path.Join(Directory.GetCurrentDirectory(), "php" + version);
            Microsoft.VisualBasic.FileIO.FileSystem.CopyDirectory(path, movedPath);

            _dbContext.PhpVersions.Add(new PhpVersion
            {
                Version = version,
                Path = movedPath
            });

            _dbContext.SaveChanges();
            Console.WriteLine($"Added version {version}");
        }

        
    }
}
