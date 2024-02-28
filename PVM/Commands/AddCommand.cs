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

        public class Folders
        {
            public string Source { get; private set; }
            public string Target { get; private set; }

            public Folders(string source, string target)
            {
                Source = source;
                Target = target;
            }
        }

        private static void CopyDirectory(string source, string target)
        {
            var stack = new Stack<Folders>();
            stack.Push(new Folders(source, target));

            while (stack.Count > 0)
            {
                var folders = stack.Pop();
                Directory.CreateDirectory(folders.Target);
                foreach (var file in Directory.GetFiles(folders.Source, "*.*"))
                {
                    File.Copy(file, Path.Combine(folders.Target, Path.GetFileName(file)));
                }

                foreach (var folder in Directory.GetDirectories(folders.Source))
                {
                    stack.Push(new Folders(folder, Path.Combine(folders.Target, Path.GetFileName(folder))));
                }
            }
        }
    }
}
