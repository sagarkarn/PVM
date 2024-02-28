using Microsoft.EntityFrameworkCore;
using PVM.Data;

namespace PVM.Commands
{
    public class ListCommand
    {
        private readonly SqliteDbContext _dbContext;

        public ListCommand(SqliteDbContext dbContext)
        {
            _dbContext = dbContext;
            _dbContext.Database.Migrate();
        }

        public void List()
        {
            Console.WriteLine("Current working directory: " + System.AppContext.BaseDirectory);
            var phpVersions = _dbContext.PhpVersions.ToList();
            if (phpVersions.Count == 0)
            {
                Console.WriteLine("No versions found");
                return;
            }

            foreach (var version in phpVersions)
            {
                Console.WriteLine($"{version.Version} - {version.Path} {(version.IsCurrent ? "(current)" : "")}");
            }
        }
    }
}
