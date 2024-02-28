using System;
using System.Collections.Generic;
using System.ComponentModel.DataAnnotations;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace PVM.Data
{
    public class PhpVersion
    {
        [Key]
        public int Id { get; set; }
        public string Version { get; set; }
        public string Path { get; set; }
        public bool IsCurrent { get; set; }
    }
}
