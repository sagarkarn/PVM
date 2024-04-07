using Cocona;
using HtmlAgilityPack;
using Microsoft.EntityFrameworkCore;
using PVM.Data;
using PVM.helpers;
using System.ComponentModel;
using System.IO.Compression;

namespace PVM.Commands
{
    public enum Type
    {
        Nts,
        Ts
    }
    public class InstallCommand
    {
        private readonly SqliteDbContext _dbContext;

        public InstallCommand(SqliteDbContext dbContext)
        {
            _dbContext = dbContext;
            _dbContext.Database.Migrate();
        }

        public async void Install([Argument] string version, [Description("Only nts and ts value is allowed")] Type type = Type.Nts)
        {
            var phpVersion = _dbContext.PhpVersions.FirstOrDefault(v => v.Version.StartsWith(version));

            if (phpVersion != null)
            {
                Console.WriteLine($"Version {phpVersion.Version} already exists");
                return;
            }

            UpdateList();

            var arch = Environment.Is64BitOperatingSystem ? "x64" : "x86";

            var installUrl = _dbContext.InstallUrls.FirstOrDefault(u => u.Version.StartsWith(version) && u.Architecture == arch && u.Type == type.ToString().ToLower());

            if (installUrl == null)
            {
                Console.WriteLine($"Download url not found for version {version}");
                return;
            }

            var url = installUrl.Url;

            var path = Path.Join(Directory.GetCurrentDirectory(), "php" + installUrl.Version);
            var zipPath = Path.Join(Directory.GetCurrentDirectory(), "php" + installUrl.Version + ".zip");

            if (File.Exists(zipPath))
            {
                File.Delete(zipPath);
            }

            Console.WriteLine($"Downloading... 0%");

            using (var client = new HttpClientDownloadWithProgress(url, zipPath))
            {
                client.ProgressChanged += (totalFileSize, totalBytesDownloaded, progressPercentage) =>
                {
                    ClearLastLine();
                    Console.WriteLine($"downloading... {progressPercentage}%");
                };

                var awaitResponse = client.StartDownload();
                awaitResponse.Wait();

            }

            if (Directory.Exists(path))
            {
                Directory.Delete(path, true);
            }
            ZipFile.ExtractToDirectory(zipPath, path);

            UpdateIniFile(path);

            var phpVersions = _dbContext.PhpVersions.Where(x => x.Version == installUrl.Version).ToList();

            if (phpVersions.Count > 0)
            {
                phpVersions.ForEach((x) =>
                {
                    _dbContext.PhpVersions.Remove(x);
                });
            }

            _dbContext.SaveChanges();

            _dbContext.PhpVersions.Add(new PhpVersion
            {
                Version = installUrl.Version,
                Path = path
            });

            _dbContext.SaveChanges();
            Console.WriteLine($"installed successfully {installUrl.Version}");
        }

        [Ignore]
        public static void UpdateIniFile(string path)
        {
            var iniPath = Path.Join(path, "php.ini");
            if(!File.Exists(iniPath))
            {
                var devPath = Path.Join(path, "php.ini-development");
                File.Copy(devPath, iniPath);
            }

            var ini = File.ReadAllLines(iniPath).ToList();
            var extensionDir = ini.FirstOrDefault(x => x.StartsWith("extension_dir") || x.StartsWith(";extension_dir"));
            var extensionDirIndex = ini.IndexOf(extensionDir);
            ini[extensionDirIndex] = $"extension_dir = \"ext\"";

            var extension = ini.FirstOrDefault(x => x.StartsWith(";extension=curl"));
            var extensionIndex = ini.IndexOf(extension);
            ini[extensionIndex] = "extension=curl";

            File.WriteAllLines(iniPath, ini);
        }

        [Ignore]
        public static void ClearLastLine()
        {
            Console.SetCursorPosition(0, Console.CursorTop);
            Console.Write(new string(' ', Console.BufferWidth));
            Console.SetCursorPosition(0, Console.CursorTop - 1);
        }
        [Ignore]
        private void UpdateList()
        {
            using (HttpClient client = new())
            {
                client.DefaultRequestHeaders.Add("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.0.4664.110 Safari/537.36");
                var response = client.GetAsync("https://windows.php.net/downloads/releases/").Result;
                response.EnsureSuccessStatusCode();
                var html = response.Content.ReadAsStringAsync().Result;
                var doc = new HtmlDocument();
                doc.LoadHtml(html);
                var nodes = doc.DocumentNode.SelectNodes("//a");
                foreach (var node in nodes)
                {
                    var href = node.GetAttributeValue("href", "");
                    if (href.Contains("php-") && !(href.Contains("debug") || href.Contains("devel") || href.Contains("test")))
                    {
                        var version = href.Split("-")[1];
                        if (!_dbContext.InstallUrls.Any(u => u.Version == version))
                        {
                            _dbContext.InstallUrls.Add(new InstallUrl
                            {
                                Version = version,
                                Url = "https://windows.php.net" + href,
                                Architecture = href.Contains("x64") ? "x64" : "x86",
                                Type = href.Contains("nts") ? "nts" : "ts"
                            });
                        }
                    }
                }
                _dbContext.SaveChanges();
            }
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
