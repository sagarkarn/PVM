using Cocona;
using Microsoft.EntityFrameworkCore;
using PVM.Data;

namespace PVM.Commands
{
    public class ExtCommand
    {
        private readonly SqliteDbContext _dbContext;

        public ExtCommand(SqliteDbContext dbContext)
        {
            _dbContext = dbContext;
            _dbContext.Database.Migrate();
        }
        [Command("Open extension folder in file explorer of current php version.")]
        public void Ext(string? version)
        {
            PhpVersion phpVersion = _dbContext.PhpVersions.FirstOrDefault(v => v.IsCurrent);
            if (!string.IsNullOrEmpty(version))
            {
                phpVersion = _dbContext.PhpVersions.FirstOrDefault(v => v.Version == version);
            }

            
            if (phpVersion == null)
            {
                Console.WriteLine($"Version {version} not found");
                return;
            }

            var extPath = Path.Join(phpVersion.Path, "ext");
            if (!Directory.Exists(extPath))
            {
                Console.WriteLine("ext not found");
                return;
            }

            System.Diagnostics.Process.Start("explorer.exe", extPath);
        }
    }
}
