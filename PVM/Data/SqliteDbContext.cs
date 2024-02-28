using Microsoft.EntityFrameworkCore;

namespace PVM.Data
{
    public class SqliteDbContext:DbContext
    {
        public SqliteDbContext(DbContextOptions<SqliteDbContext> options) : base(options)
        {
        }
        public DbSet<PhpVersion> PhpVersions { get; set; }
        public DbSet<InstallUrl> InstallUrls { get; set; }
    }
}
