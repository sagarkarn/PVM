using Cocona;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;
using PVM.Commands;
using PVM.Data;
Directory.SetCurrentDirectory(System.AppContext.BaseDirectory);
var builder = CoconaApp.CreateBuilder();

builder.Services.AddDbContext<SqliteDbContext>(options =>
{
    options.UseSqlite("Data Source=pvm.db");
});

builder.Logging.AddFilter("Microsoft.EntityFrameworkCore", LogLevel.Warning);

var app = builder.Build();

app.AddCommands<UseCommand>();
app.AddCommands<AddCommand>();
app.AddCommands<ListCommand>();
app.AddCommands<IniCommand>();
app.AddCommands<ExtCommand>();
app.AddCommands<InstallCommand>();

app.Run();