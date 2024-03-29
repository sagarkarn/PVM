﻿using System;
using System.Collections.Generic;
using System.ComponentModel.DataAnnotations;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace PVM.Data
{
    public class InstallUrl
    {
        [Key]
        public int Id { get; set; }
        public string Version { get; set; }
        public string Url { get; set; }
        public string Type { get; set; }
        public string Architecture { get; set; }
    }
}
