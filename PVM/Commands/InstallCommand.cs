using Cocona;
using Microsoft.EntityFrameworkCore;
using PVM.Data;
using System;
using System.Collections.Generic;
using System.IO.Compression;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace PVM.Commands
{
    public class InstallCommand
    {
        private readonly SqliteDbContext _dbContext;

        public InstallCommand(SqliteDbContext dbContext)
        {
            _dbContext = dbContext;
            _dbContext.Database.Migrate();
        }

        public void Install([Argument]string version)
        {
            var phpVersion = _dbContext.PhpVersions.FirstOrDefault(v => v.Version == version);
            if (phpVersion != null)
            {
                Console.WriteLine($"Version {version} already exists");
                return;
            }

            //var installUrl = _dbContext.InstallUrls.FirstOrDefault(u => u.version == version);
            var url = "https://windows.php.net/downloads/releases/php-8.3.3-nts-Win32-vs16-x64.zip";
            //var url = "https://windows.php.net/downloads/releases/php-" + version + "-nts-Win32-VC16-x64.zip";

            var path = Path.Join(Directory.GetCurrentDirectory(), "php" + version);
            var zipPath = Path.Join(Directory.GetCurrentDirectory(), "php" + version + ".zip");

            if(File.Exists(zipPath))
            {
                File.Delete(zipPath);
            }

            Console.WriteLine($"Downloading...");

            using (HttpClient client = new())
            {
                client.DefaultRequestHeaders.Add("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.0.4664.110 Safari/537.36");
                var response = client.GetAsync(url).Result;
                response.EnsureSuccessStatusCode();
                using var stream = response.Content.ReadAsStream();
                using var fileStream = new FileStream(zipPath, FileMode.CreateNew);
                
                stream.CopyToAsync(fileStream).Wait();
            }
            if(Directory.Exists(path))
            {
                Directory.Delete(path, true);
            }
            ZipFile.ExtractToDirectory(zipPath, path);

            _dbContext.PhpVersions.Add(new PhpVersion
            {
                Version = version,
                Path = path
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
